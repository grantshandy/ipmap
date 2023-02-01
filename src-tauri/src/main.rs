#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::thread;

use pcap::{Active, Capture, Device};
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
            let capture = match get_capture() {
                Ok(cap) => cap,
                Err(err) => {
                    MessageDialogBuilder::new(
                        "Ipmap Error",
                        format!("Initialization Error: {err}"),
                    )
                    .kind(MessageDialogKind::Error)
                    .buttons(MessageDialogButtons::Ok)
                    .show();
                    return Err(err.into());
                }
            };

            WindowBuilder::new(&app.app_handle(), "ipmap", WindowUrl::App("/".parse().unwrap()))
                .title("ipmap")
                .center()
                .build()
                .expect("failed to start about window");

            Ok(())
        })
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
