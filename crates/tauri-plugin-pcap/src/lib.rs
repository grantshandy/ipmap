//! Linking to a self-extracting libpcap-linking child executable :)

use tauri::{
    Manager, Runtime,
    plugin::{Builder, TauriPlugin},
};

use crate::model::PcapState;

pub(crate) mod child;
pub mod commands;
pub mod model;

const PLUGIN_NAME: &str = "pcap";

/// Initialize the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let builder = builder::<R>();

    Builder::new(PLUGIN_NAME)
        .invoke_handler(builder.invoke_handler())
        .setup(move |app, _api| {
            app.manage(PcapState::default());
            builder.mount_events(app);
            Ok(())
        })
        .build()
}

fn builder<R: Runtime>() -> tauri_specta::Builder<R> {
    tauri_specta::Builder::<R>::new()
        .plugin_name(PLUGIN_NAME)
        .events(tauri_specta::collect_events![model::PcapStateChange])
        .commands(tauri_specta::collect_commands![
            commands::start_capture::<tauri::Wry>,
            commands::stop_capture,
            commands::init_pcap::<tauri::Wry>,
            commands::traceroute_enabled::<tauri::Wry>,
            commands::run_traceroute::<tauri::Wry>,
            commands::print_error,
        ])
}

#[cfg(test)]
mod test {
    use super::*;
    use child_ipc::ErrorKind;
    use specta::{Generics, Type, TypeCollection, datatype::DataType};
    use std::fs;

    const BINDINGS_PATH: &str = "./guest-js/bindings.ts";

    #[test]
    fn export_types() {
        builder::<tauri::Wry>()
            .constant("PCAP_ERROR_KINDS", pcap_error_kinds())
            .error_handling(tauri_specta::ErrorHandlingMode::Result)
            .export(
                specta_typescript::Typescript::default()
                    .bigint(specta_typescript::BigIntExportBehavior::Number),
                BINDINGS_PATH,
            )
            .unwrap();

        // Remove duplicate TAURI_CHANNEL type from bindings.ts
        fs::write(
            BINDINGS_PATH,
            fs::read_to_string(BINDINGS_PATH)
                .unwrap()
                .lines()
                .filter(|line| line.trim() != "export type TAURI_CHANNEL<TSend> = null")
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .unwrap();
    }

    fn pcap_error_kinds() -> Vec<String> {
        let DataType::Enum(e) =
            ErrorKind::inline(&mut TypeCollection::default(), Generics::Definition)
        else {
            unreachable!();
        };

        e.variants()
            .iter()
            .map(|(name, _)| name.to_string())
            .collect()
    }
}
