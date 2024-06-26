#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{net::IpAddr, path::PathBuf, process};

use dashmap::DashMap;
use tauri::{
    api::dialog::{blocking::MessageDialogBuilder, MessageDialogKind},
    async_runtime, Manager,
};

mod capture;

mod analyze;
mod geoip;

mod expiry_set;

/// The cached result of public_ip::addr()
type PublicIpAddress = IpAddr;

type DatabaseState = DashMap<PathBuf, geoip::database::Database>;

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .manage(DatabaseState::default())
        .setup(|app| {
            // TODO: make optional and asynchronous in the background instead of blocking the main thread.
            let Some(ip) = async_runtime::block_on(public_ip::addr()) else {
                MessageDialogBuilder::new(
                    "Ipmap Error",
                    "unable to detect your public ip address.",
                )
                .kind(MessageDialogKind::Error)
                .show();

                process::exit(1);
            };

            app.manage(ip);

            Ok(())
        })
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
