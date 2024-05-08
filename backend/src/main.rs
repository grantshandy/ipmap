#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod geoip;

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .manage(geoip::RuntimeDb::default())
        .manage(capture::CaptureState::default())
        .invoke_handler(tauri::generate_handler![
            capture::list_devices,
            capture::set_device,
            capture::start_capturing,
            capture::stop_capturing,
            geoip::set_database,
            geoip::lookup_ip
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

