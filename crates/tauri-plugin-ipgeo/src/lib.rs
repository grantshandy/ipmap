//! Database management for IP geolocation archives.
//!
//! This module provides abstractions for loading, caching, selecting, and managing
//! multiple IP geolocation databases. It supports memory-mapped, checksummed archives
//! and exposes APIs for querying location and coordinate data by IP address.
//!
//! The main entry point is [`DbState`], which tracks all loaded databases and
//! coordinates their lifecycle and selection state.

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod archive;
pub mod commands;
mod disk;
mod model;
mod my_loc;

pub use {
    disk::{DatabaseSource, DiskArchive, DynamicDatabase},
    model::{DbState, DbStateInfo},
    my_loc::get as try_get_my_location,
};

const PLUGIN_NAME: &str = "ipgeo";

/// Initialize the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let builder = builder::<R>();

    Builder::new(PLUGIN_NAME)
        .invoke_handler(builder.invoke_handler())
        .setup(move |app, _api| {
            app.manage(DbState::new(app.app_handle())?);
            builder.mount_events(app);
            Ok(())
        })
        .build()
}

fn builder<R: Runtime>() -> tauri_specta::Builder<R> {
    tauri_specta::Builder::<R>::new()
        .plugin_name(PLUGIN_NAME)
        .events(tauri_specta::collect_events![model::DbStateChange])
        .commands(tauri_specta::collect_commands![
            commands::refresh_cache::<tauri::Wry>,
            commands::download_source::<tauri::Wry>,
            commands::unload_database::<tauri::Wry>,
            commands::set_selected_database::<tauri::Wry>,
            commands::database_state,
            commands::lookup_ip,
            commands::lookup_dns,
            commands::lookup_host,
            commands::my_location
        ])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn export_types() {
        builder::<tauri::Wry>()
            .error_handling(tauri_specta::ErrorHandlingMode::Result)
            .export(
                specta_typescript::Typescript::default(),
                "./guest-js/bindings.ts",
            )
            .unwrap();
    }
}
