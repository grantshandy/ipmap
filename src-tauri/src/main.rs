#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{net::IpAddr, thread, time::Duration};

use etherparse::{InternetSlice, SlicedPacket};
use pcap::Device;
use tauri::{
    generate_context, generate_handler, AppHandle, Builder, CustomMenuItem, Manager, Menu,
    MenuItem, Submenu, WindowBuilder, WindowUrl,
};

fn main() {
    Builder::default()
        .menu(
            Menu::new()
                .add_submenu(Submenu::new(
                    "File",
                    Menu::new().add_native_item(MenuItem::Quit),
                ))
                .add_submenu(Submenu::new(
                    "Help",
                    Menu::new().add_item(CustomMenuItem::new("about", "About")),
                )),
        )
        .on_menu_event(|event| match event.menu_item_id() {
            "about" => {
                if event.window().get_window("about").is_none() {
                    thread::spawn(move || about_window(event.window().app_handle()));
                }
            }
            _ => (),
        })
        .invoke_handler(generate_handler![start_polling])
        .run(generate_context!())
        .expect("open application window");
}

fn about_window(handle: AppHandle) {
    WindowBuilder::new(
        &handle,
        "about",
        WindowUrl::App("about.html".parse().unwrap()),
    )
    .resizable(false)
    .inner_size(320.0, 380.0)
    .title("About")
    .menu(Menu::default())
    .center()
    .build()
    .expect("open about window");
}

#[derive(Clone, Debug, serde::Serialize)]
struct Connection {
    pub city: String,
    pub lat: f64,
    pub lon: f64,
    pub ip: IpAddr,
}

#[tauri::command]
fn start_polling(handle: AppHandle) {
    thread::spawn(move || {
        let device: Device = Device::lookup()
            .expect("failed to look up device")
            .expect("no device found");
        let mut cap = device.open().expect("failed to open capture");

        loop {
            let packet = match cap.next_packet() {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("error getting packet: {err}");
                    continue;
                }
            };

            let ip: IpAddr = match SlicedPacket::from_ethernet(&packet) {
                Ok(value) => {
                    let ip = match match value.ip {
                        Some(data) => data,
                        None => continue,
                    } {
                        InternetSlice::Ipv4(ip, _) => IpAddr::V4(ip.source_addr()),
                        InternetSlice::Ipv6(ip, _) => IpAddr::V6(ip.source_addr()),
                    };

                    if !ip_rfc::global(&ip) {
                        continue;
                    }

                    ip
                }
                Err(err) => {
                    eprintln!("error parsing packet: {err}");
                    continue;
                }
            };

            println!("{:?}", ip);

            // handle.emit_all("connection", Connection {
            //   city: String::new(),
            //     lat: 0,
            //     lon: 0,
            //     ip: IpAddr::
            // }).unwrap();
        }
    });
}
