#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{net::IpAddr, path::PathBuf, sync::Arc};
use tauri::async_runtime::Mutex;

use dashmap::DashMap;

mod capture;

mod analyze;
mod geoip;

mod expiry_set;

/// The cached result of public_ip::addr()
type PublicIpAddress = Arc<Mutex<Option<Result<IpAddr, String>>>>;

type DatabaseState = DashMap<PathBuf, geoip::database::Database>;

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .manage(DatabaseState::new())
        .manage(PublicIpAddress::default())
        .invoke_handler(tauri::generate_handler![
            capture::list_devices,
            capture::start_capturing,
            capture::stop_capturing,
            geoip::load_database,
            geoip::list_databases,
            geoip::lookup_ip,
            geoip::my_location,
            analyze::dns_lookup_addr,
            analyze::validate_ip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}