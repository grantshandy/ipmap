use std::{net::IpAddr, path::PathBuf, sync::atomic::Ordering, time::Duration};

use ipgeo::{Database, LookupInfo};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{State, ipc::Channel};

use crate::store::{
    DatabaseStore, DatabaseStoreInfo, NamedDatabase,
    sources::{self, DatabaseSource},
};

const DNS_LOOKUP_TIMEOUT: Duration = Duration::from_millis(300);

#[tauri::command]
#[specta::specta]
pub async fn init_cache(store: State<'_, DatabaseStore>) -> Result<(), String> {
    store.init_cache().map_err(|e| e.to_string())?;

    store.emit_update();

    Ok(())
}

#[derive(Copy, Clone, Type, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuiltinDatabaseSources {
    DbIp,
    Geolite2,
}

impl Into<DatabaseSource> for BuiltinDatabaseSources {
    fn into(self) -> DatabaseSource {
        match self {
            BuiltinDatabaseSources::DbIp => sources::DBIP_CITY,
            BuiltinDatabaseSources::Geolite2 => sources::GEOLITE2_CITY,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn download(
    store: State<'_, DatabaseStore>,
    source: BuiltinDatabaseSources,
    stage: Channel<&'static str>,
    progress: Channel<f32>,
) -> Result<(), String> {
    store
        .download(
            source.into(),
            |p| {
                let _ = progress.send(p);
            },
            |s| {
                let _ = stage.send(s);
            },
        )
        .await
        .map_err(|e| e.to_string())?;

    store.emit_update();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn load_file(store: State<'_, DatabaseStore>, path: PathBuf) -> Result<(), String> {
    let db = tokio::task::spawn_blocking(move || NamedDatabase::from_file(path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    store.insert(db);

    store.emit_update();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn database_info(store: State<'_, DatabaseStore>) -> DatabaseStoreInfo {
    store.inner().info()
}

#[tauri::command]
#[specta::specta]
pub fn set_selected(store: State<'_, DatabaseStore>, name: String) {
    store.selected.store(
        store
            .loaded
            .iter()
            .find(|kv| &name == kv.value().metadata.display_name)
            .map(|kv| *kv.key())
            .unwrap_or(0),
        Ordering::Relaxed,
    );
    store.emit_update();
}

#[tauri::command]
#[specta::specta]
pub fn lookup_ip(store: State<'_, DatabaseStore>, ip: IpAddr) -> Option<LookupInfo> {
    store.get(ip)
}

/// Get a hostname with the system for a given IP address.
#[tauri::command]
#[specta::specta]
pub async fn lookup_dns(ip: IpAddr) -> Result<Option<String>, ()> {
    let host = async { dns_lookup::lookup_addr(&ip).ok() };

    tokio::time::timeout(DNS_LOOKUP_TIMEOUT, host)
        .await
        .map_err(|_| ())
}

/// Get a hostname with the system for a given IP address.
#[tauri::command]
#[specta::specta]
pub async fn lookup_host(host: &str) -> Result<Option<IpAddr>, ()> {
    let ip = async {
        dns_lookup::lookup_host(host)
            .ok()
            .and_then(|i| i.first().copied())
    };

    tokio::time::timeout(DNS_LOOKUP_TIMEOUT, ip)
        .await
        .map_err(|_| ())
}
