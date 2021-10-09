#![feature(ip)]

#[macro_use]
extern crate clap;

use std::thread;

use pcap::Device;
use clap::Arg;

mod ip;
mod ui;

#[tokio::main]
async fn main() {
    let app = app_from_crate!()
        .arg(
            Arg::with_name("show-interfaces")
                .long("show-interfaces")
                .short("n")
                .help("Show network interfaces")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("interface")
                .long("interface")
                .short("i")
                .help("Select what interface to capture packets from.")
                .takes_value(true),
        )
        .get_matches();

    if app.is_present("show-interfaces") {
        let device_list = Device::list().expect("My bad from rust.");

        for device in device_list {
            println!("{}: {}", device.name, device.desc.unwrap_or("".to_string()));
        }

        std::process::exit(0);
    }

    #[cfg(unix)]
    let cap = Device::lookup().expect("My bad from rust.");
    #[cfg(windows)]
    let cap = user_select_device();

    let cap = cap.open().expect("My bad from rust.");

    thread::spawn(|| {
        ui::web_view();
    });

    ip::manage_ip(cap).await;
}

#[cfg(windows)]
fn user_select_device() -> Device {
    let mut devices = Device::list().expect("My bad from rust.");
    if devices.is_empty() {
        println!("Found no device to listen on, maybe you need to run as an Adminstrator");
        std::process::exit(1);
    }
    println!("Select which device to listen on: (choose the number of the item)");
    for (i, d) in devices.iter().enumerate() {
        println!("{}: {:?}", i, d);
    }
    use std::io;

    let mut input = String::new();
    let n = loop {
        io::stdin().read_line(&mut input).expect("My bad from rust.");
        match input.trim().parse() {
            Ok(n) => {
                if n < devices.len() {
                    break n;
                } else {
                    println!("Invalid choice, try again");
                    input.clear();
                }
            }
            Err(_) => {
                println!("Invalid choice, try again");
                input.clear();
            }
        }
    };
    println!("Listening on {:?}", devices[n]);
    devices.remove(n)
}