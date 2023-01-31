#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::thread;

use pcap::Device;
use tauri::{
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
        .invoke_handler(tauri::generate_handler![poll_connections])
        .run(tauri::generate_context!())
        .expect("error starting application window");
}

fn about_window(handle: AppHandle) {
    WindowBuilder::new(
        &handle,
        "About",
        WindowUrl::App("about.html".parse().unwrap()),
    )
    .inner_size(300.0, 300.0)
    .resizable(false)
    .title("About")
    .menu(Menu::default())
    .center()
    .build()
    .expect("error starting about window");
}

#[tauri::command]
fn poll_connections(handle: AppHandle) {
    println!("getting connections");

    let device: Device = match Device::lookup() {
        Ok(device) => match device {
            Some(device) => device,
            None => {
                handle
                    .emit_all("error", Some("No network adapter found".to_string()))
                    .unwrap();
                return;
            }
        },
        Err(err) => {
            handle
                .emit_all(
                    "error",
                    Some(format!("Failed to lookup network adapter: {:?}", err)),
                )
                .unwrap();
            return;
        }
    };

    let mut capture = match device.open() {
        Ok(capture) => capture,
        Err(err) => {
            handle
                .emit_all(
                    "error",
                    Some(format!("Failed to open network adapter capture: {:?}", err)),
                )
                .unwrap();
            return;
        }
    };

    println!("all good");
}
