use std::{net::IpAddr, path::PathBuf, thread};

use ipgeo::{Database, GenericDatabase, LookupInfo};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

use crate::{DNS_LOOKUP_TIMEOUT, DbState, DbStateChange, DbStateInfo};

#[tauri::command]
#[specta::specta]
#[cfg_attr(not(db_preloads), allow(unused_variables))]
pub async fn load_internals(app: AppHandle, state: State<'_, DbState>) -> Result<(), String> {
    #[cfg(db_preloads)]
    crate::preloads::load_builtins(&state);

    DbStateChange::emit(&app);

    Ok(())
}

/// Load a IP-Geolocation database into the program from the filename.
#[tauri::command]
#[specta::specta]
pub fn load_database(app: AppHandle, path: PathBuf) {
    thread::spawn(move || {
        if let Err(err) = load_database_internal(app.clone(), app.state(), path) {
            tracing::error!("Error Loading Database: {err}");
            app.dialog()
                .message(&err)
                .title("Error Loading Database")
                .buttons(MessageDialogButtons::Ok)
                .blocking_show();
        }
    });
}

fn load_database_internal(
    app: AppHandle,
    state: State<'_, DbState>,
    path: PathBuf,
) -> Result<(), String> {
    if state.ipv4_db.exists(&path) || state.ipv6_db.exists(&path) {
        return Err("Database with the same name already loaded".to_string());
    }

    if state.loading.read().expect("read loading").is_some() {
        return Err("Database is already loading".to_string());
    }

    state
        .loading
        .write()
        .expect("write loading")
        .replace(path.clone());
    DbStateChange::emit(&app);

    tracing::info!("loading database from {path:?}");

    let db = ipgeo::detect(&path).map_err(|e| e.to_string())?;

    match db {
        GenericDatabase::Ipv4(db) => state.ipv4_db.insert(&path, db),
        GenericDatabase::Ipv6(db) => state.ipv6_db.insert(&path, db),
    }

    tracing::info!("finished loading database {path:#?}");

    state.loading.write().expect("write loading").take();
    DbStateChange::emit(&app);

    Ok(())
}

/// Unload the database, freeing up memory.
#[tauri::command]
#[specta::specta]
pub async fn unload_database(
    app: AppHandle,
    state: State<'_, DbState>,
    path: PathBuf,
) -> Result<(), String> {
    tracing::info!("unloading database {path:#?}");

    state.ipv4_db.remove(&path);
    state.ipv6_db.remove(&path);

    DbStateChange::emit(&app);

    Ok(())
}

/// Set the given database as the selected database for lookups.
#[tauri::command]
#[specta::specta]
pub fn set_selected_database(app: AppHandle, state: State<'_, DbState>, path: PathBuf) {
    tracing::info!("set selected database as {path:#?}");

    state.ipv4_db.set_selected(&path);
    state.ipv6_db.set_selected(&path);

    DbStateChange::emit(&app);
}

/// Retrieve the current state of the database.
/// This info is given out in [`DbStateChange`], but this is useful for getting it at page load, for example.
#[tauri::command]
#[specta::specta]
pub fn database_state(state: State<'_, DbState>) -> DbStateInfo {
    state.info()
}

/// Lookup a given IP address in the currently selected database(s).
#[tauri::command]
#[specta::specta]
pub fn lookup_ip(state: State<'_, DbState>, ip: IpAddr) -> Option<LookupInfo> {
    state.get(ip)
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

/// Attempt to get the user's current location
#[tauri::command]
#[specta::specta]
pub async fn my_location(state: State<'_, DbState>) -> Result<LookupInfo, String> {
    match crate::my_loc::get().await? {
        (_, Some(info)) => Ok(info),
        (ip, None) => match lookup_ip(state, ip) {
            Some(info) => Ok(info),
            None => Err(format!("Your IP {ip} not found in loaded database")),
        },
    }
}
