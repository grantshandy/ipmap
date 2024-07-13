//! Corresponding definitions in /frontend/src/bindings/index.ts

use std::{net::IpAddr, path::PathBuf, sync::Arc};

use crate::{LoadedIpv4Databases, LoadedIpv6Databases, PublicIpAddress};
use database::{Coordinate, Database, DatabaseInfo, IpRange, Ipv4Bytes, Ipv6Bytes, LocationInfo};
use tauri::State;

pub mod database {
    include!(concat!(env!("OUT_DIR"), "/internal_database.rs"));
}

fn db_v4(
    databases: State<'_, LoadedIpv4Databases>,
    database: Option<PathBuf>,
) -> Result<Arc<Database<Ipv4Bytes>>, String> {
    match database {
        Some(path) => databases
            .get(&path)
            .ok_or("IPv4 database path not loaded".to_string())
            .map(|db| db.value().clone()),
        None => database::IPV4_DATABASE
            .as_ref()
            .ok_or("no internal ipv4 database set".to_string())
            .cloned(),
    }
}

fn db_v6(
    databases: State<'_, LoadedIpv6Databases>,
    database: Option<PathBuf>,
) -> Result<Arc<Database<Ipv6Bytes>>, String> {
    match database {
        Some(path) => databases
            .get(&path)
            .ok_or("IPv6 database path not loaded".to_string())
            .map(|db| db.value().clone()),
        None => database::IPV6_DATABASE
            .as_ref()
            .ok_or("no internal ipv6 database set".to_string())
            .cloned(),
    }
}

/// Load a database by its identifier (a path).
/// No path (None) is for the database optionally compiled into the executable.
#[tauri::command]
pub async fn load_database(
    ipv4_databases: State<'_, LoadedIpv4Databases>,
    ipv6_databases: State<'_, LoadedIpv6Databases>,
    path: Option<PathBuf>,
) -> Result<Option<DatabaseInfo>, String> {
    match path {
        Some(path) => {
            let path_str = path.to_string_lossy();

            let info = if path_str.contains("ipv4") {
                tracing::info!("reading ipv4 db at {path:?}");
                let db = database::Database::<Ipv4Bytes>::from_csv(&path, None)
                    .map_err(|e| e.to_string())?;

                let info = db.get_db_info();
                ipv4_databases.insert(path, db.into());
                info
            } else if path_str.contains("ipv6") {
                tracing::info!("reading ipv6 db at {path:?}");
                let db = database::Database::<Ipv6Bytes>::from_csv(&path, None)
                    .map_err(|e| e.to_string())?;

                let info = db.get_db_info();
                ipv6_databases.insert(path, db.into());
                info
            } else {
                return Err("database filename must include ipv4 or ipv6, sorry.".to_string());
            };

            Ok(Some(info))
        }
        None => {
            if database::IPV4_DATABASE.is_none() {
                tracing::warn!("no internal ipv4 database set");
            }

            Ok(database::IPV4_DATABASE.as_ref().map(|db| db.get_db_info()))
        }
    }
}

/// Delete a database from the global state, freeing up memory
#[tauri::command]
pub async fn unload_database(
    databases: State<'_, LoadedIpv4Databases>,
    path: PathBuf,
) -> Result<(), String> {
    tracing::info!("unloading {path:?} database");

    databases.remove(&path);

    Ok(())
}

/// List all databases (by info)
#[tauri::command]
pub async fn list_databases(
    ipv4_databases: State<'_, LoadedIpv4Databases>,
    ipv6_databases: State<'_, LoadedIpv6Databases>,
) -> Result<Vec<DatabaseInfo>, ()> {
    let mut databases: Vec<DatabaseInfo> = ipv4_databases.iter().map(|v| v.get_db_info()).collect();

    databases.extend(ipv6_databases.iter().map(|v| v.get_db_info()));

    if let Some(internal) = database::IPV4_DATABASE.as_ref() {
        databases.insert(0, internal.get_db_info());
    }

    if let Some(internal) = database::IPV6_DATABASE.as_ref() {
        databases.insert(0, internal.get_db_info());
    }

    tracing::info!("{databases:#?}");

    Ok(databases)
}

/// Lookup the coordinate for an IP address in the database
#[tauri::command]
pub async fn lookup_ip(
    ipv4_databases: State<'_, LoadedIpv4Databases>,
    ipv6_databases: State<'_, LoadedIpv6Databases>,
    database: Option<PathBuf>,
    ip: IpAddr,
) -> Result<Option<Coordinate>, String> {
    if !ip_rfc::global(&ip) {
        return Err(format!("ip {ip} is not global"));
    }

    match ip {
        IpAddr::V4(ip) => db_v4(ipv4_databases, database).map(|db| db.get(ip)),
        IpAddr::V6(ip) => db_v6(ipv6_databases, database).map(|db| db.get(ip)),
    }
}

/// Find the range in the database for a given IP
#[tauri::command]
pub async fn lookup_ip_range(
    databases_v4: State<'_, LoadedIpv4Databases>,
    databases_v6: State<'_, LoadedIpv6Databases>,
    database: Option<PathBuf>,
    ip: IpAddr,
) -> Result<IpRange, String> {
    if !ip_rfc::global(&ip) {
        return Err(format!("ip {ip} is not global"));
    }

    let range = match ip {
        IpAddr::V4(ip) => db_v4(databases_v4, database)?.get_range(ip),
        IpAddr::V6(ip) => db_v6(databases_v6, database)?.get_range(ip),
    };

    let Some(range) = range else {
        return Err(format!("ip {ip} not found in database"));
    };

    Ok(range)
}

/// Finds the block of ips for a given coordinate in the database
#[tauri::command]
pub async fn lookup_ip_blocks(
    databases_v4: State<'_, LoadedIpv4Databases>,
    databases_v6: State<'_, LoadedIpv6Databases>,
    database: Option<PathBuf>,
    coord: Coordinate,
) -> Result<Vec<IpRange>, String> {
    let mut res = db_v4(databases_v4, database.clone())
        .map(|db| db.get_ranges(&coord))
        .unwrap_or_default();
    res.extend(
        db_v6(databases_v6, database)
            .map(|db| db.get_ranges(&coord))
            .unwrap_or_default(),
    );
    Ok(res)
}

/// The nearest location in the database from a given coordinate
#[tauri::command]
pub async fn nearest_location(
    databases_v4: State<'_, LoadedIpv4Databases>,
    databases_v6: State<'_, LoadedIpv6Databases>,
    database: Option<PathBuf>,
    coord: Coordinate,
) -> Result<Coordinate, String> {
    Ok(db_v4(databases_v4, database.clone())
        .map(|db| db.nearest_location(&coord))
        .unwrap_or(db_v6(databases_v6, database).map(|db| db.nearest_location(&coord))?))
}

/// Associated City, State, and Country for a Coordinate
#[tauri::command]
pub fn location_info(
    databases_v4: State<'_, LoadedIpv4Databases>,
    databases_v6: State<'_, LoadedIpv6Databases>,
    database: Option<PathBuf>,
    coord: Coordinate,
) -> Result<Option<LocationInfo>, String> {
    Ok(db_v4(databases_v4, database.clone())
        .map(|db| db.get_location_info(&coord))
        .unwrap_or(db_v6(databases_v6, database).map(|db| db.get_location_info(&coord))?))
}

/// Our coordinate based on the current database
#[tauri::command]
pub async fn my_location(
    databases_v4: State<'_, LoadedIpv4Databases>,
    databases_v6: State<'_, LoadedIpv6Databases>,
    ip: State<'_, PublicIpAddress>,
    database: Option<PathBuf>,
) -> Result<Coordinate, String> {
    lookup_ip(databases_v4, databases_v6, database, *ip)
        .await
        .and_then(|loc| {
            loc.ok_or(format!(
                "no location found for your public ip address {}",
                *ip
            ))
        })
}

/// Lookup the associated DNS address with a string.
#[tauri::command]
pub async fn dns_lookup_addr(ip: IpAddr) -> Option<String> {
    dns_lookup::lookup_addr(&ip).ok()
}

/// Validate if a string is a global IP address.
#[tauri::command]
pub async fn validate_ip(ip: String) -> bool {
    ip.parse::<IpAddr>().is_ok_and(|ip| ip_rfc::global(&ip))
}
