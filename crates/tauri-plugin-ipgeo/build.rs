const COMMANDS: &[&str] = &[
    "refresh_cache",
    "download_source",
    "unload_database",
    "set_selected_database",
    "database_state",
    "lookup_ip",
    "lookup_dns",
    "lookup_host",
    "my_location",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
