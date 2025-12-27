use std::{
    borrow::Cow,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::{Path, PathBuf},
    thread,
};

use ipgeo::{
    CombinedDatabase, Database, GenericDatabase, LookupInfo, SingleDatabase,
    download::CombinedDatabaseSource,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{
    AppHandle, Manager, State,
    async_runtime::{self, Sender},
    ipc::Channel,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

use crate::db::{DNS_LOOKUP_TIMEOUT, DbState, DbStateChange, DbStateInfo, DynamicDatabase};

macro_rules! ip_location_db {
    ($path:literal) => {
        std::borrow::Cow::Borrowed(concat!(
            "https://raw.githubusercontent.com/sapics/ip-location-db/refs/heads/main/",
            $path
        ))
    };
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseSource {
    DbIpCombined,
    Geolite2Combined,
    SingleCsvGz {
        is_ipv6: bool,
        url: String,
        is_num: bool,
    },
    CombinedCsvGz {
        ipv4: String,
        ipv6: String,
        is_num: bool,
    },
    File(PathBuf),
}

fn filename_guess(path: &str) -> String {
    path.rsplit_once("/")
        .unwrap_or(("", "unknown"))
        .1
        .to_string()
}

/// Download a combined database
#[tauri::command]
#[specta::specta]
pub async fn download_source(
    handle: AppHandle,
    state: State<'_, DbState>,
    source: DatabaseSource,
    name_resp: Channel<String>,
    prog_resp: Channel<f64>,
) -> Result<(), String> {
    tracing::info!("downloading {source:?}");

    let name = match &source {
        DatabaseSource::DbIpCombined => "DB-IP City Combined".to_string(),
        DatabaseSource::Geolite2Combined => "Geolite2 City Combined".to_string(),
        DatabaseSource::SingleCsvGz { url, .. } => filename_guess(url),
        DatabaseSource::CombinedCsvGz { ipv4, ipv6, .. } => {
            format!("{}/{}", filename_guess(&ipv4), filename_guess(&ipv6))
        }
        DatabaseSource::File(path) => path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown File".to_string()),
    };

    let _ = name_resp.send(name.clone());

    let db = match download_source_internal(prog_resp, source).await {
        Ok(db) => db,
        Err(err) => {
            let err = format!("failed to download database: {err}");
            tracing::error!("{err}");
            return Err(err);
        }
    };

    match db {
        DynamicDatabase::Combined(db) => state.combined.insert(name, db),
        DynamicDatabase::Generic(GenericDatabase::Ipv4(db)) => state.ipv4_db.insert(name, db),
        DynamicDatabase::Generic(GenericDatabase::Ipv6(db)) => state.ipv6_db.insert(name, db),
    }

    state.emit_info(&handle);

    Ok(())
}

async fn download_source_internal(
    progress_sender: Channel<f64>,
    source: DatabaseSource,
) -> anyhow::Result<DynamicDatabase> {
    let cb = move |val: usize, max: usize| {
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
        DatabaseSource::CombinedCsvGz { ipv4, ipv6, is_num } => {
            let src = CombinedDatabaseSource {
                ipv4_csv_url: Cow::Owned(ipv4),
                ipv6_csv_url: Cow::Owned(ipv6),
                is_num,
            };

            CombinedDatabase::download(src, cb)
                .await
                .map(DynamicDatabase::Combined)?
        }
        DatabaseSource::SingleCsvGz {
            is_ipv6,
            url,
            is_num,
        } => match is_ipv6 {
            true => SingleDatabase::<Ipv6Addr>::download(url, is_num, cb)
                .await
                .map(GenericDatabase::Ipv6)
                .map(DynamicDatabase::Generic)?,
            false => SingleDatabase::<Ipv4Addr>::download(url, is_num, cb)
                .await
                .map(GenericDatabase::Ipv4)
                .map(DynamicDatabase::Generic)?,
        },
        DatabaseSource::File(path) => tokio::task::spawn_blocking(move || ipgeo::detect(&path))
            .await?
            .map(DynamicDatabase::Generic)?,
    };

    Ok(db)
}

/// Unload the database, freeing up memory.
#[tauri::command]
#[specta::specta]
pub fn unload_database(app: AppHandle, state: State<'_, DbState>, name: String) {
    tracing::info!("unloading database {name:#?}");

    state.ipv4_db.remove(&name);
    state.ipv6_db.remove(&name);
    state.combined.remove(&name);

    state.emit_info(&app);
}

/// Set the given database as the selected database for lookups.
#[tauri::command]
#[specta::specta]
pub async fn set_selected_database(
    app: AppHandle,
    state: State<'_, DbState>,
    name: String,
) -> Result<(), String> {
    tracing::info!("set selected database as {name:#?}");

    state.ipv4_db.set_selected(&name);
    state.ipv6_db.set_selected(&name);
    state.combined.set_selected(&name);

    state.emit_info(&app);

    Ok(())
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
            .and_then(|mut i| i.next())
    };

    tokio::time::timeout(DNS_LOOKUP_TIMEOUT, ip)
        .await
        .map_err(|_| ())
}

/// Attempt to get the user's current location
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
