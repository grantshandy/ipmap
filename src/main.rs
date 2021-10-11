#![windows_subsystem = "windows"]
#[macro_use]
extern crate clap;

use pcap::Device;
use std::thread;

mod ip;
mod ui;

#[tokio::main]
async fn main() {
    app_from_crate!().get_matches();

    #[cfg(unix)]
    // We autodetect by default on unix like badasses
    let cap = Device::lookup().unwrap();
    #[cfg(windows)]
    // uhhh saddd windows has to select it on their own...
    let cap = ui::windows_select_device();

    println!(
        "Capturing on {}",
        cap.desc.clone().unwrap_or("Unknown Device".to_string())
    );

    let cap = cap.open().unwrap();

    thread::spawn(|| {
        ui::web_view();
    });

    ip::manage_ip(cap).await;
}
