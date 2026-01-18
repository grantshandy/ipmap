#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

pub fn main() {
    tracing_subscriber::fmt::init();

    let builder = builder();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_ipgeo::init())
        .plugin(tauri_plugin_pcap::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn builder() -> tauri_specta::Builder {
    tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
        commands::open_about_window::<tauri::Wry>,
        commands::platform
    ])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn export_types() {
        builder()
            .constant("APP_VERSION", env!("CARGO_PKG_VERSION"))
            .error_handling(tauri_specta::ErrorHandlingMode::Result)
            .export(
                specta_typescript::Typescript::default()
                    .bigint(specta_typescript::BigIntExportBehavior::Number),
                "bindings/raw.ts",
            )
            .unwrap();
    }
}
