#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod analyze;
mod capture;
mod geoip;

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            capture::list_devices,
            capture::start_capturing,
            capture::stop_capturing,
            geoip::load_internal_database,
            geoip::lookup_ip,
            analyze::dns_lookup_addr,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
