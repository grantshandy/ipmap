//! Corresponding definitions in /frontend/src/bindings/index.ts

use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    sync::Arc,
};

use crate::{LoadedDatabases, PublicIpAddress};
use tauri::State;

pub mod database {
    include!(concat!(env!("OUT_DIR"), "/internal_database.rs"));
}

use database::{Coordinate, Database, DatabaseInfo, IpRange, LocationInfo};

fn db(
    databases: State<'_, LoadedDatabases>,
    database: Option<PathBuf>,
) -> Result<Arc<Database>, String> {
    match database {
        Some(path) => databases
            .get(&path)
            .ok_or("database path not loaded".to_string())
            .map(|db| db.value().clone()),
        None => database::DATABASE
            .as_ref()
            .ok_or("no internal database set".to_string())
            .cloned(),
    }
}

/// Load a database by its identifier (a path).
/// No path (None) is for the database optionally compiled into the executable.
#[tauri::command]
pub async fn load_database(
    databases: State<'_, LoadedDatabases>,
    path: Option<PathBuf>,
) -> Result<Option<DatabaseInfo>, String> {
    match path {
        Some(path) => {
            tracing::info!("reading db at {path:?}");

            let db = database::Database::from_csv(&path, None).map_err(|e| e.to_string())?;
            let info = db.get_db_info();

            databases.insert(path, db.into());

            Ok(Some(info))
        }
        None => {
            if database::DATABASE.is_none() {
                tracing::warn!("no internal database set");
            }

            Ok(database::DATABASE.as_ref().map(|db| db.get_db_info()))
        }
    }
}

/// Delete a database from the global state, freeing up memory
#[tauri::command]
pub async fn unload_database(
    databases: State<'_, LoadedDatabases>,
    path: PathBuf,
) -> Result<(), String> {
    tracing::info!("unloading {path:?} database");

    databases.remove(&path);

    Ok(())
}

/// List all databases (by info)
#[tauri::command]
pub async fn list_databases(
    databases: State<'_, LoadedDatabases>,
) -> Result<Vec<DatabaseInfo>, ()> {
    let mut databases: Vec<DatabaseInfo> = databases.iter().map(|v| v.get_db_info()).collect();

    if let Some(internal) = database::DATABASE.as_ref() {
        databases.insert(0, internal.get_db_info());
    }

    Ok(databases)
}

/// Lookup the coordinate for an IP address in the database
#[tauri::command]
pub async fn lookup_ip(
    databases: State<'_, LoadedDatabases>,
    database: Option<PathBuf>,
    ip: Ipv4Addr,
) -> Result<Option<Coordinate>, String> {
    if !ip_rfc::global_v4(&ip) {
        return Err(format!("ip {ip} is not global"));
    }

    db(databases, database).map(|db| db.get(ip))
}

/// Find the range in the database for a given IP
#[tauri::command]
pub async fn lookup_ip_range(
    databases: State<'_, LoadedDatabases>,
    database: Option<PathBuf>,
    ip: Ipv4Addr,
) -> Result<IpRange, String> {
    if !ip_rfc::global_v4(&ip) {
        return Err(format!("ip {ip} is not global"));
    }

    let range = db(databases, database)?.get_range(ip);

    let Some(range) = range else {
        return Err(format!("ip {ip} not found in database"));
    };

    Ok(range.into())
}

/// Finds the block of ips for a given coordinate in the database
#[tauri::command]
pub async fn lookup_ip_blocks(
    databases: State<'_, LoadedDatabases>,
    database: Option<PathBuf>,
    coord: Coordinate,
) -> Result<Vec<IpRange>, String> {
    db(databases, database).map(|db| db.get_ranges(&coord))
}

/// The nearest location in the database from a given coordinate
#[tauri::command]
pub async fn nearest_location(
    databases: State<'_, LoadedDatabases>,
    database: Option<PathBuf>,
    coord: Coordinate,
) -> Result<Coordinate, String> {
    db(databases, database).map(|db| db.nearest_location(&coord))
}

/// Associated City, State, and Country for a Coordinate
#[tauri::command]
pub fn location_info(
    databases: State<'_, LoadedDatabases>,
    database: Option<PathBuf>,
    coord: Coordinate,
) -> Result<Option<LocationInfo>, String> {
    db(databases, database).map(|db| db.get_location_info(&coord))
}

/// Our coordinate based on the current database
#[tauri::command]
pub async fn my_location(
    databases: State<'_, LoadedDatabases>,
    ip: State<'_, PublicIpAddress>,
    database: Option<PathBuf>,
) -> Result<Coordinate, String> {
    let IpAddr::V4(ip) = *ip else {
        return Err("IPv6 addresses not yet supported".to_string());
    };

    lookup_ip(databases, database, ip)
        .await
        .and_then(|loc| loc.ok_or(format!("no location found for your public ip address {ip}")))
}

/// Lookup the associated DNS address with a string.
#[tauri::command]
pub async fn dns_lookup_addr(ip: IpAddr) -> Option<String> {
    dns_lookup::lookup_addr(&ip).ok()
}

/// Validate if a string is a global IPv4 address.
#[tauri::command]
pub async fn validate_ip(ip: String) -> bool {
    ip.parse::<Ipv4Addr>()
        .is_ok_and(|ip| ip_rfc::global_v4(&ip))
}
