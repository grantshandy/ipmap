use std::{net::Ipv4Addr, path::PathBuf};

use crate::DatabaseState;
use tauri::State;

pub mod database {
    include!(concat!(env!("OUT_DIR"), "/internal_database.rs"));
}

use database::{Database, DatabaseInfo};

#[tauri::command]
pub async fn lookup_ip(
    databases: State<'_, DatabaseState>,
    database: Option<PathBuf>,
    ip: Ipv4Addr,
) -> Result<Option<database::Location>, String> {
    if !ip_rfc::global_v4(&ip) {
        return Err(format!("ip {ip} is not global"));
    }

    let res = match database {
        Some(path) => databases
            .get(&path)
            .ok_or("database not found".to_string())?
            .value()
            .get(ip),
        None => database::DATABASE
            .as_ref()
            .ok_or("no internal database set")?
            .get(ip),
    };

    Ok(res)
}

/// Load a database by its identifier (a path).
/// No path (None) is for the database optionally compiled into the executable.
#[tauri::command]
pub async fn load_database(
    databases: State<'_, DatabaseState>,
    path: Option<PathBuf>,
) -> Result<Option<DatabaseInfo>, String> {
    match path {
        Some(path) => {
            tracing::info!("reading db at {path:?}");

            let db = database::Database::from_csv(&path, None).map_err(|e| e.to_string())?;
            let info = db.info();

            databases.insert(path, db);

            Ok(Some(info))
        }
        None => {
            if database::DATABASE.is_none() {
                tracing::warn!("no internal database set");
            }

            Ok(database::DATABASE.as_ref().map(Database::info))
        }
    }
}

/// List all databases (by info)
#[tauri::command]
pub async fn list_databases(databases: State<'_, DatabaseState>) -> Result<Vec<DatabaseInfo>, ()> {
    tracing::info!("listing databases");

    let mut databases: Vec<DatabaseInfo> = databases.iter().map(|v| v.info()).collect();

    if let Some(internal) = database::DATABASE.as_ref() {
        databases.push(internal.info());
    }

    Ok(databases)
}
