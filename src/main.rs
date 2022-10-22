use std::{io, process};

use actix_web::{get, http::header::ContentType, web, App, HttpResponse, HttpServer};
use argh::FromArgs;
use log::info;
use pcap::Device;
use tokio::{
    signal,
    sync::watch::{self, Receiver},
};

mod ip_broadcast;
mod ip_capture;
mod ui;
mod geolocate;

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
    ip_rx: Receiver<String>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Args = argh::from_env();

    pretty_env_logger::init();

    handle_exit().await;

    // a channel for communicating between client "threads" and the ip capture thread
    let (ip_tx, ip_rx) = watch::channel(String::from("init"));

    // spawn a green thread to capture ip addresses and send it to the clients
    tokio::spawn(async move {
        let device = Device::lookup().unwrap().unwrap();

        ip_capture::capture(ip_tx, device).await;
    });

    // spawn the ui
    if !args.headless {
        let args_clone = args.clone();

        tokio::spawn(async move {
            ui::run_gui(&args_clone.ip, &args_clone.port);
        });
    }

    let state = AppState { ip_rx };

    info!("starting server on {}:{}", &args.ip, &args.port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(ip_broadcast::ip_stream)
            .service(index)
    })
    .bind((args.ip, args.port))?
    .run()
    .await
}

/// vitejs is setup with a plugin so that all html/js/css is put into a single file for convinience as seen here.
#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("../frontend/dist/index.html"))
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
