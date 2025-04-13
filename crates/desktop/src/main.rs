#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    pretty_env_logger::init();

    ipmap_lib::run()
}
