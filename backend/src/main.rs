#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{net::IpAddr, path::PathBuf, process, sync::Arc};

use capture_state::CaptureState;
use dashmap::DashMap;
use geoip::database::{Database, Ipv4Bytes, Ipv6Bytes};
use tauri::{
    api::dialog::{blocking::MessageDialogBuilder, MessageDialogKind},
    async_runtime, Manager,
};

mod capture;
mod capture_state;
mod geoip;

/// The cached result of public_ip::addr()
type PublicIpAddress = IpAddr;
type LoadedIpv4Databases = DashMap<PathBuf, Arc<Database<Ipv4Bytes>>>;
type LoadedIpv6Databases = DashMap<PathBuf, Arc<Database<Ipv6Bytes>>>;

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .manage(LoadedIpv4Databases::default())
        .manage(LoadedIpv6Databases::default())
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
            geoip::lookup_ip_blocks,
            geoip::nearest_location,
            geoip::location_info,
            geoip::dns_lookup_addr,
            geoip::validate_ip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
