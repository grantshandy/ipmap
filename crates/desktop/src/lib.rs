use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    let ts_export_builder = tauri_specta::Builder::<tauri::Wry>::new()
        .events(tauri_specta::collect_events![
            ipgeo_state::DbStateChange,
            pcap_state::PcapStateChange
        ])
        .commands(tauri_specta::collect_commands![
            ipgeo_state::commands::load_database,
            ipgeo_state::commands::unload_database,
            ipgeo_state::commands::database_state,
            ipgeo_state::commands::set_selected_database,
            ipgeo_state::commands::lookup_ip,
            ipgeo_state::commands::lookup_dns,
            ipgeo_state::commands::lookup_host,
            ipgeo_state::commands::my_location,
            pcap_state::commands::init_pcap,
            pcap_state::commands::start_capture,
            pcap_state::commands::stop_capture,
            pcap_state::commands::traceroute_enabled,
            pcap_state::commands::run_traceroute,
            pcap_state::commands::platform,
        ]);

    #[cfg(all(debug_assertions, not(mobile)))]
    ts_export_builder
        .export(
            specta_typescript::Typescript::default()
                .bigint(specta_typescript::BigIntExportBehavior::Number),
            "../../ui/src/bindings/raw.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(ts_export_builder.invoke_handler())
        .setup(move |app| {
            ts_export_builder.mount_events(app);

            app.manage(ipgeo_state::DbState::default());
            app.manage(pcap_state::PcapState::new());

            #[cfg(debug_assertions)]
            app.get_webview_window("main").unwrap().open_devtools();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
