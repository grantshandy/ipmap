#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use dashmap::DashMap;

mod analyze;
mod capture;
mod geoip;

type DatabaseState = DashMap<PathBuf, geoip::database::Database>;

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .manage(DatabaseState::new())
        .invoke_handler(tauri::generate_handler![
            capture::list_devices,
            capture::start_capturing,
            capture::stop_capturing,
            geoip::load_database,
            geoip::list_databases,
            geoip::lookup_ip,
            analyze::dns_lookup_addr
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
