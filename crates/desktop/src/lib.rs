use tauri::Manager;

mod db;
mod pcap;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    let ts_export_builder = tauri_specta::Builder::<tauri::Wry>::new()
        .events(tauri_specta::collect_events![
            db::DbStateChange,
            pcap::PcapStateChange
        ])
        .commands(tauri_specta::collect_commands![
            utils::open_about_window,
            db::commands::download_source,
            db::commands::unload_database,
            db::commands::database_state,
            db::commands::set_selected_database,
            db::commands::lookup_ip,
            db::commands::lookup_dns,
            db::commands::lookup_host,
            db::commands::my_location,
            pcap::commands::init_pcap,
            pcap::commands::start_capture,
            pcap::commands::stop_capture,
            pcap::commands::traceroute_enabled,
            pcap::commands::run_traceroute,
            pcap::commands::print_error,
        ])
        .constant("PCAP_ERROR_KINDS", utils::pcap_error_kinds())
        .constant("PLATFORM", utils::Platform::current())
        .constant("APP_VERSION", env!("CARGO_PKG_VERSION"));

    // TODO: export in build script
    #[cfg(all(debug_assertions, not(mobile)))]
    ts_export_builder
        .export(
            specta_typescript::Typescript::default()
                .bigint(specta_typescript::BigIntExportBehavior::Number),
            "../../ui/src/lib/bindings/raw.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(ts_export_builder.invoke_handler())
        .setup(move |app| {
            ts_export_builder.mount_events(app);

            app.manage(db::DbState::new(app.handle())?);
            app.manage(pcap::PcapState::default());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
