#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{Builder, Menu, Submenu, MenuItem};

fn main() {
    Builder::default()
        .menu(Menu::new().add_submenu(Submenu::new(
            "File",
            Menu::new().add_native_item(MenuItem::Quit),
        )))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
