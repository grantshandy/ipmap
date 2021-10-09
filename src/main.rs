#![windows_subsystem = "windows"]
#[macro_use]
extern crate clap;

use std::thread;

mod ip;
mod ui;

#[tokio::main]
async fn main() {
    let _ = app_from_crate!().get_matches();

    #[cfg(unix)]
    // Autodetect cuz on unix we're badasses
    let cap = Device::lookup().unwrap();
    #[cfg(windows)]
    // uhhh saddd windows has to select it on their own...
    let cap = ui::windows_select_device();

    println!("Capturing on {}\n", cap.desc.clone().unwrap_or("Unknown Device".to_string()));

    let cap = cap.open().unwrap();

    thread::spawn(|| {
        ui::web_view();
    });

    ip::manage_ip(cap).await;
}