use tauri::Manager;

mod db_state;
mod pcap;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri_specta::Builder::<tauri::Wry>::new()
        .events(tauri_specta::collect_events![
            db_state::DatabaseStateChange,
            pcap::PcapStateChange,
            pcap::NewPacket
        ])
        .commands(tauri_specta::collect_commands![
            db_state::load_database,
            db_state::unload_database,
            db_state::database_state,
            db_state::set_selected_database,
            db_state::lookup_ip,
            pcap::pcap_state,
            pcap::start_capture,
            pcap::stop_capture
        ]);

    #[cfg(all(debug_assertions, not(mobile)))]
    builder
        .export(
            specta_typescript::Typescript::default().bigint(specta_typescript::BigIntExportBehavior::Number),
            "../../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            app.manage(db_state::GlobalDatabaseState::default());
            app.manage(pcap::GlobalPcapState::default());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
