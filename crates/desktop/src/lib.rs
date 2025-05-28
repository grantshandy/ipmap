use tauri::Manager;

mod db_state;
mod pcap_state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // #[cfg(debug_assertions)] // only enable instrumentation in development builds
    // let devtools = tauri_plugin_devtools::init();

    let ts_export_builder = tauri_specta::Builder::<tauri::Wry>::new()
        .events(tauri_specta::collect_events![
            db_state::DatabaseStateChange,
            pcap_state::PcapStateChange,
        ])
        .commands(tauri_specta::collect_commands![
            db_state::load_database,
            db_state::unload_database,
            db_state::database_state,
            db_state::set_selected_database,
            db_state::lookup_ip,
            pcap_state::sync_pcap_state,
            pcap_state::start_capture,
            pcap_state::stop_capture
        ]);

    #[cfg(all(debug_assertions, not(mobile)))]
    ts_export_builder
        .export(
            specta_typescript::Typescript::default()
                .bigint(specta_typescript::BigIntExportBehavior::Number),
            "../../ui/src/bindings/raw.ts",
        )
        .expect("Failed to export typescript bindings");

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(ts_export_builder.invoke_handler())
        .setup(move |app| {
            ts_export_builder.mount_events(app);

            app.manage(db_state::GlobalDatabaseState::default());
            app.manage(pcap_state::GlobalPcapState::default());

            #[cfg(debug_assertions)]
            app.get_webview_window("main").unwrap().open_devtools();

            Ok(())
        });

    // #[cfg(debug_assertions)]
    // {
    //     builder = builder.plugin(devtools);
    // }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
