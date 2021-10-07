#![feature(ip)]

use std::thread;

mod ui;
mod ip;

#[tokio::main]
async fn main() {
    thread::spawn(|| {
        ui::web_view();
    });

    ip::manage_ip().await;
}