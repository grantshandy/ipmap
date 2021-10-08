#![feature(ip)]

#[macro_use]
extern crate clap;

use std::thread;

mod ui;
mod ip;

#[tokio::main]
async fn main() {
    app_from_crate!().get_matches();

    thread::spawn(|| {
        ui::web_view();
    });

    ip::manage_ip().await;
}