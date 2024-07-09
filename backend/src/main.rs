#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{net::IpAddr, path::PathBuf, process};

use capture_state::CaptureState;
use dashmap::DashMap;
use tauri::{
    api::dialog::{blocking::MessageDialogBuilder, MessageDialogKind},
    async_runtime, Manager,
};

mod capture;
mod capture_state;

mod analyze;
mod geoip;

/// The cached result of public_ip::addr()
type PublicIpAddress = IpAddr;

type Global = DashMap<PathBuf, geoip::database::Database>;

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .manage(Global::default())
        .manage(CaptureState::default())
        .setup(|app| {
            tracing::info!("getting ip");

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

            tracing::info!("got ip");

            app.manage(ip);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            capture::list_devices,
            capture::start_capturing,
            capture::stop_capturing,
            capture::all_connections,
            capture::current_connections,
            geoip::load_database,
            geoip::unload_database,
            geoip::list_databases,
            geoip::lookup_ip,
            geoip::my_location,
            geoip::lookup_ip_range,
            geoip::nearest_location,
            analyze::dns_lookup_addr,
            analyze::validate_ip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
