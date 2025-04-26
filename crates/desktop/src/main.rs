#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tracing_subscriber::fmt().init();
    tracing::info!("Starting Ipmap");

    ipmap_lib::run()
}
