use std::{io, process};

use actix_web::{web, App, HttpServer};
use argh::FromArgs;
use log::info;
use pcap::Device;
use tokio::{
    signal,
    sync::watch::{self, Receiver},
};

mod geolocate;
mod ip_broadcast;
mod ip_capture;
mod static_pages;
mod ui;

pub const STREAM_KEEP_ALIVE_SECS: u64 = 3;

/// See who you're connecting to!
#[derive(FromArgs, Clone)]
struct Args {
    /// what port to serve the web server on
    #[argh(option, short = 'p', default = "5171")]
    port: u16,
    /// what ip to serve the web server on
    #[argh(option, short = 'i', default = "String::from(\"127.0.0.1\")")]
    ip: String,
    /// don't open the web view gui
    #[argh(switch, short = 'h')]
    headless: bool,
}

/// State that actix_web, the web server, holds onto during excecution
#[derive(Clone)]
pub struct AppState {
    stream_rx: Receiver<String>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Args = argh::from_env();
    pretty_env_logger::init();
    handle_exit().await;

    // a channel for communicating between client "threads" and the ip capture thread
    let (stream_tx, stream_rx) = watch::channel("init".to_string());

    // spawn a green thread to capture ip addresses and send it to the clients
    tokio::spawn(async move {
        let device = Device::lookup().unwrap().unwrap();

        ip_capture::capture(stream_tx, device).await;
    });

    // spawn the gui if wanted
    if !args.headless {
        let args_clone = args.clone();

        tokio::spawn(async move {
            ui::run_gui(&args_clone.ip, &args_clone.port);
        });
    }

    let state = AppState { stream_rx };

    info!("starting server on http://{}:{}", &args.ip, &args.port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(ip_broadcast::ip_stream)
            .service(static_pages::index)
            .service(static_pages::marker_icon)
            .service(static_pages::marker_shadow)
    })
    .bind((args.ip, args.port))?
    .run()
    .await
}

/// Make sure we absolutely exit the program on Ctrl+C.
///
/// While testing I struggled to close the program sometimes
/// so this makes me feel better.
async fn handle_exit() {
    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();

        info!("Exiting!");
        process::exit(0);
    });
}
