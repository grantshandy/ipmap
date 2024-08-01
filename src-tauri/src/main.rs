#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{net::IpAddr, path::PathBuf, process, sync::Arc};

use capture_state::CaptureState;
use dashmap::DashMap;
use ipdb_city::{Database, DatabaseInfo, DatabaseQuery, DatabaseType, Ipv4Bytes, Ipv6Bytes};
use tauri::{async_runtime, AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder, MessageDialogKind};

mod capture;
mod capture_state;
mod geoip;
mod traceroute;

mod internal_database {
    include!(concat!(env!("OUT_DIR"), "/internal_database.rs"));
}

/// The cached result of public_ip::addr()
type PublicIpAddress = IpAddr;

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .manage(GlobalDatabases::default())
        .manage(CaptureState::default())
        .setup(|app| {
            tracing::info!("getting ip");

            // TODO: make optional and asynchronous in the background instead of blocking the main thread.
            let Some(ip) = async_runtime::block_on(public_ip::addr()) else {
                MessageDialogBuilder::new(
                    app.dialog().clone(),
                    "Ipmap Error",
                    "unable to detect your public ip address.",
                )
                .kind(MessageDialogKind::Error)
                .blocking_show();

                process::exit(1)
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
            geoip::dns_lookup_host,
            geoip::validate_ip,
            traceroute::traceroute,
            traceroute::is_privileged,
            about_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Default)]
struct GlobalDatabases {
    ipv4: DashMap<DatabaseType, Arc<Database<Ipv4Bytes>>>,
    ipv6: DashMap<DatabaseType, Arc<Database<Ipv6Bytes>>>,
}

impl GlobalDatabases {
    /// TODO: only load once
    pub fn init_internal(&self) {
        if let Some(internal) = internal_database::IPV4_DATABASE.as_ref() {
            self.ipv4.insert(DatabaseType::Internal, internal.clone());
        }

        if let Some(internal) = internal_database::IPV6_DATABASE.as_ref() {
            self.ipv6.insert(DatabaseType::Internal, internal.clone());
        }
    }

    pub fn get(&self, query: &DatabaseQuery) -> DatabaseResult {
        DatabaseResult {
            ipv4: query
                .ipv4
                .as_ref()
                .and_then(|t| self.ipv4.get(t))
                .map(|kv| kv.value().clone()),
            ipv6: query
                .ipv6
                .as_ref()
                .and_then(|t| self.ipv6.get(t))
                .map(|kv| kv.value().clone()),
        }
    }

    pub fn databases(&self) -> Vec<DatabaseInfo> {
        let mut infos = Vec::new();

        infos.extend(self.ipv4.iter().map(|db| db.get_db_info()));
        infos.extend(self.ipv6.iter().map(|db| db.get_db_info()));

        infos
    }

    pub fn remove(&self, path: PathBuf) {
        self.ipv4.remove(&DatabaseType::Loaded(path.clone()));
        self.ipv6.remove(&DatabaseType::Loaded(path));
    }

    pub fn insert_ipv4(&self, path: PathBuf, db: Database<Ipv4Bytes>) {
        self.ipv4.insert(DatabaseType::Loaded(path), Arc::new(db));
    }

    pub fn insert_ipv6(&self, path: PathBuf, db: Database<Ipv6Bytes>) {
        self.ipv6.insert(DatabaseType::Loaded(path), Arc::new(db));
    }
}

#[derive(Clone, Debug)]
struct DatabaseResult {
    ipv4: Option<Arc<Database<Ipv4Bytes>>>,
    ipv6: Option<Arc<Database<Ipv6Bytes>>>,
}

#[tauri::command]
async fn about_window<R: Runtime>(handle: AppHandle<R>, theme: String) {
    if let Some(window) = handle.get_webview_window("about") {
        window.set_focus().expect("bring about window to focus");

        return;
    }

    let Some(main) = handle.get_webview_window("main") else {
        return;
    };

    WebviewWindowBuilder::new(&handle, "about", WebviewUrl::App("about.html".into()))
        .minimizable(false)
        .maximizable(false)
        .resizable(false)
        .inner_size(500.0, 450.0)
        .title("About")
        .center()
        .initialization_script(&format!(r#"window.theme = "{theme}";"#))
        .parent(&main)
        .expect("set about parent window")
        .build()
        .expect("failed to build window");
}
