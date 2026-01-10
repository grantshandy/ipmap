//! Accessors to the runtime [`DbState`] for the frontend UI.

use std::{net::IpAddr, path::PathBuf};

use super::{DNS_LOOKUP_TIMEOUT, DatabaseSource, DbState, DbStateInfo, DynamicDatabase};

use ipgeo::{CombinedDatabase, Database, LookupInfo, download::CombinedDatabaseSource};
use tauri::{AppHandle, State, ipc::Channel};

macro_rules! ip_location_db {
    ($path:literal) => {
        // TODO: change back
        std::borrow::Cow::Borrowed(concat!("http://localhost:8000/", $path))
    };
}

/// Load in the databases from the disk cache.
#[tauri::command]
#[specta::specta]
pub async fn refresh_cache(handle: AppHandle, state: State<'_, DbState>) -> Result<(), String> {
    tracing::debug!("refreshing cache");

    state.refresh_cache().await.map_err(|e| e.to_string())?;
    state.emit_info(&handle);

    Ok(())
}

/// Load a [`DatabaseSource`] from its origin.
#[tauri::command]
#[specta::specta]
pub async fn download_source(
    handle: AppHandle,
    state: State<'_, DbState>,
    source: DatabaseSource,
    name_resp: Channel<&str>,
    prog_resp: Channel<f64>,
) -> Result<(), String> {
    tracing::info!("downloading {source:?}");

    let name = source.to_string();

    let _ = name_resp.send(&name);

    let db = match download_source_internal(prog_resp, &source).await {
        Ok(db) => db,
        Err(err) => {
            let err = format!("failed to download database: {err}");
            tracing::error!("{err}");
            return Err(err);
        }
    };

    state.insert(source, db).await.map_err(|e| {
        tracing::error!("error adding database: {e}");
        e.to_string()
    })?;
    state.emit_info(&handle);

    Ok(())
}

async fn download_source_internal(
    progress_sender: Channel<f64>,
    source: &DatabaseSource,
) -> anyhow::Result<DynamicDatabase> {
    let cb = move |val: u64, max: u64| {
        println!("{val}/{max}");
        let _ = progress_sender.send(val as f64 / max as f64);
    };

    let db = match source {
        DatabaseSource::DbIpCombined => {
            let src = CombinedDatabaseSource {
                ipv4_csv_url: ip_location_db!("dbip-city/dbip-city-ipv4-num.csv.gz"),
                ipv6_csv_url: ip_location_db!("dbip-city/dbip-city-ipv6-num.csv.gz"),
                is_num: true,
            };

            CombinedDatabase::download(src, cb)
                .await
                .map(DynamicDatabase::Combined)?
        }
        DatabaseSource::Geolite2Combined => {
            let src = CombinedDatabaseSource {
                ipv4_csv_url: ip_location_db!("geolite2-city/geolite2-city-ipv4-num.csv.gz"),
                ipv6_csv_url: ip_location_db!("geolite2-city/geolite2-city-ipv6-num.csv.gz"),
                is_num: true,
            };

            CombinedDatabase::download(src, cb)
                .await
                .map(DynamicDatabase::Combined)?
        }
        DatabaseSource::File(path) => {
            let path = PathBuf::from(path);

            tokio::task::spawn_blocking(move || ipgeo::detect(&path))
                .await?
                .map(DynamicDatabase::Generic)?
        }
    };

    Ok(db)
}

/// Unload the database, freeing up memory.
#[tauri::command]
#[specta::specta]
pub fn unload_database(app: AppHandle, state: State<'_, DbState>, source: DatabaseSource) {
    tracing::info!("unloading database {source:?}");

    state.remove(&source);
    state.emit_info(&app);
}

/// Set the given [`DatabaseSource`] as the selected database
/// for lookups on it's associated database type.
#[tauri::command]
#[specta::specta]
pub async fn set_selected_database(
    app: AppHandle,
    state: State<'_, DbState>,
    source: DatabaseSource,
) -> Result<(), String> {
    tracing::info!("set selected database as {source:?}");

    state.set_selected(&source);
    state.emit_info(&app);

    Ok(())
}

/// Retrieve the current [`DbStateInfo`] of the database.
///
/// This info is given out in [`DbStateChange`](super::DbStateChange),
/// but this is useful for getting it at page load, for example.
#[tauri::command]
#[specta::specta]
pub fn database_state(state: State<'_, DbState>) -> DbStateInfo {
    state.info()
}

/// Lookup a given [`IpAddr`] in the currently selected database(s).
#[tauri::command]
#[specta::specta]
pub fn lookup_ip(state: State<'_, DbState>, ip: IpAddr) -> Option<LookupInfo> {
    state.get(ip)
}

/// Get a hostname with the system for a given [`IpAddr`].
#[tauri::command]
#[specta::specta]
pub async fn lookup_dns(ip: IpAddr) -> Result<Option<String>, ()> {
    let host = async { dns_lookup::lookup_addr(&ip).ok() };

    tokio::time::timeout(DNS_LOOKUP_TIMEOUT, host)
        .await
        .map_err(|_| ())
}

/// Get a hostname with the system for a given [`IpAddr`].
#[tauri::command]
#[specta::specta]
pub async fn lookup_host(host: &str) -> Result<Option<IpAddr>, ()> {
    let ip = async {
        dns_lookup::lookup_host(host)
            .ok()
            .and_then(|mut i| i.next())
    };

    tokio::time::timeout(DNS_LOOKUP_TIMEOUT, ip)
        .await
        .map_err(|_| ())
}

/// Attempt to get the user's current [`LookupInfo`] from their IP address.
#[tauri::command]
#[specta::specta]
pub async fn my_location(state: State<'_, DbState>) -> Result<LookupInfo, String> {
    match crate::db::my_loc::get().await? {
        (_, Some(info)) => Ok(info),
        (ip, None) => match lookup_ip(state, ip) {
            Some(info) => Ok(info),
            None => Err(format!("Your IP {ip} not found in loaded database")),
        },
    }
}
