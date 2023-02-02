#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{env, fs, net::IpAddr, sync::Arc, thread};

use etherparse::{InternetSlice, SlicedPacket};
use pcap::{Active, Capture, Device};
use sled::Db;
use tauri::{
    api::dialog::{blocking::MessageDialogBuilder, MessageDialogButtons, MessageDialogKind},
    AppHandle, Builder, CustomMenuItem, Manager, Menu, MenuItem, Submenu, WindowBuilder, WindowUrl,
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
            "about" if event.window().get_window("about").is_none() => {
                thread::spawn(move || about_window(event.window().app_handle()));
            }
            _ => (),
        })
        .setup(|app| {
            // start pcap
            let capture = match get_capture() {
                Ok(cap) => cap,
                Err(err) => {
                    MessageDialogBuilder::new("Pcap Initialization Error", &err)
                        .kind(MessageDialogKind::Error)
                        .buttons(MessageDialogButtons::Ok)
                        .show();
                    return Err(err.into());
                }
            };

            // start sled cache db
            let db = match get_database() {
                Ok(db) => db,
                Err(err) => {
                    MessageDialogBuilder::new("Database Initialization Error", &err.to_string())
                        .kind(MessageDialogKind::Error)
                        .buttons(MessageDialogButtons::Ok)
                        .show();
                    return Err(err.into());
                }
            };
            app.manage(db);

            // spawn ip handler in other thread
            let handle = Arc::new(app.app_handle());
            thread::spawn(move || poll_connections(handle, capture));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![write_connection, read_connection])
        .run(tauri::generate_context!())
        .expect("failed to start main application");
}

fn about_window(handle: AppHandle) {
    WindowBuilder::new(&handle, "about", WindowUrl::App("/about".parse().unwrap()))
        .title("About Ipmap")
        .center()
        .inner_size(320.0, 240.0)
        .menu(Menu::new())
        .build()
        .expect("failed to start about window");
}

fn get_capture() -> Result<Capture<Active>, String> {
    match Device::lookup() {
        Ok(device) => match device {
            Some(device) => match device.open() {
                Ok(cap) => Ok(cap),
                Err(err) => Err(format!(
                    "Failed to capture packets on network adapter: {:?}",
                    err
                )),
            },
            None => Err("No network adapters found".to_string()),
        },
        Err(err) => Err(format!("Failed to lookup network adapters: {:?}", err)),
    }
}

fn get_database() -> sled::Result<Db> {
    let mut base_dir = dirs_next::data_dir().unwrap_or(env::current_dir()?);
    base_dir.push("ipmap");
    base_dir.push("location_cache");

    fs::create_dir_all(&base_dir)?;
    sled::open(base_dir)
}

#[derive(Copy, Clone, serde::Serialize)]
struct NewConnection {
    pub ip: IpAddr,
}

fn poll_connections(handle: Arc<AppHandle>, mut capture: Capture<Active>) {
    while let Ok(packet) = capture.next_packet() {
        match SlicedPacket::from_ethernet(&packet) {
            Ok(packet) if packet.ip.is_some() => {
                let ip: IpAddr = match packet.ip.unwrap() {
                    InternetSlice::Ipv4(ip, _) => IpAddr::V4(ip.source_addr()),
                    InternetSlice::Ipv6(ip, _) => IpAddr::V6(ip.source_addr()),
                };

                if !ip_rfc::global(&ip) {
                    continue;
                }

                handle.emit_all("connection", NewConnection { ip }).unwrap();
            }
            Err(err) => {
                eprintln!("Error parsing packet: {:?}", err);
                continue;
            }
            _ => (),
        }
    }
}

#[derive(Clone, serde::Serialize)]
struct Connection {}

#[tauri::command]
fn write_connection(handle: AppHandle) {
    
}

#[tauri::command]
fn read_connection(handle: AppHandle) -> Option<Connection> {
    None
}
