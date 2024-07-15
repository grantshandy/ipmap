//! Corresponding definitions in /frontend/src/bindings/index.ts

use std::{cmp::Ordering, net::IpAddr, path::PathBuf};

use database::{Coordinate, Database, DatabaseInfo, IpRange, Ipv4Bytes, Ipv6Bytes, LocationInfo};
use rstar::PointDistance;
use tauri::State;

use crate::{DatabaseQuery, GlobalDatabases, PublicIpAddress};

pub mod database {
    include!(concat!(env!("OUT_DIR"), "/internal_database.rs"));
}

/// Load a database by its identifier (a path).
/// No path (None) is for the database optionally compiled into the executable.
#[tauri::command]
pub async fn load_database(
    loaded_databases: State<'_, GlobalDatabases>,
    path: PathBuf,
) -> Result<DatabaseInfo, String> {
    tracing::info!("loading {path:?} database");

    match Database::<Ipv4Bytes>::from_csv(&path, None) {
        Ok(db) => {
            let info = db.get_db_info();
            loaded_databases.insert_ipv4(path, db);
            Ok(info)
        }
        Err(ipv4_err) => match Database::<Ipv6Bytes>::from_csv(&path, None) {
            Ok(db) => {
                let info = db.get_db_info();
                loaded_databases.insert_ipv6(path, db);
                Ok(info)
            }
            Err(ipv6_err) => Err(format!(
                "failed to parse as ipv4-num: {ipv4_err}, and as ipv6-num: {ipv6_err}"
            )),
        },
    }
}

/// Delete a database from the global state, freeing up memory
#[tauri::command]
pub async fn unload_database(
    loaded_databases: State<'_, GlobalDatabases>,
    path: PathBuf,
) -> Result<(), String> {
    tracing::info!("unloading {path:?} database");

    loaded_databases.remove(path);

    Ok(())
}

/// List all databases (by info)
#[tauri::command]
pub async fn list_databases(
    loaded_databases: State<'_, GlobalDatabases>,
) -> Result<Vec<DatabaseInfo>, ()> {
    loaded_databases.init_internal();

    Ok(loaded_databases.databases())
}

/// Lookup the coordinate for an IP address in the database
#[tauri::command]
pub async fn lookup_ip(
    loaded_databases: State<'_, GlobalDatabases>,
    database: DatabaseQuery,
    ip: IpAddr,
) -> Result<Option<Coordinate>, String> {
    if !ip_rfc::global(&ip) {
        return Err(format!("ip {ip} is not global"));
    }

    let coord = match ip {
        IpAddr::V4(ip) => loaded_databases
            .ipv4(&database)
            .map_err(|e| e.to_string())?
            .get(ip),
        IpAddr::V6(ip) => loaded_databases
            .ipv6(&database)
            .map_err(|e| e.to_string())?
            .get(ip),
    };

    Ok(coord)
}

/// Find the range in the database for a given IP
#[tauri::command]
pub async fn lookup_ip_range(
    loaded_databases: State<'_, GlobalDatabases>,
    database: DatabaseQuery,
    ip: IpAddr,
) -> Result<IpRange, String> {
    if !ip_rfc::global(&ip) {
        return Err(format!("ip {ip} is not global"));
    }

    let range = match ip {
        IpAddr::V4(ip) => loaded_databases
            .ipv4(&database)
            .map_err(|e| e.to_string())?
            .get_range(ip),
        IpAddr::V6(ip) => loaded_databases
            .ipv6(&database)
            .map_err(|e| e.to_string())?
            .get_range(ip),
    };

    range.ok_or(format!("ip {ip} not found in database"))
}

/// Finds the block of ips for a given coordinate in the database
#[tauri::command]
pub async fn lookup_ip_blocks(
    loaded_databases: State<'_, GlobalDatabases>,
    database: DatabaseQuery,
    coord: Coordinate,
) -> Result<Vec<IpRange>, ()> {
    let mut ranges = Vec::new();

    if let Ok(db) = loaded_databases.ipv4(&database) {
        ranges.extend(db.get_ranges(&coord));
    }

    if let Ok(db) = loaded_databases.ipv6(&database) {
        ranges.extend(db.get_ranges(&coord));
    }

    Ok(ranges)
}

/// The nearest location in the database from a given coordinate
#[tauri::command]
pub async fn nearest_location(
    loaded_databases: State<'_, GlobalDatabases>,
    database: DatabaseQuery,
    coord: Coordinate,
) -> Result<Coordinate, String> {
    match (
        loaded_databases
            .ipv4(&database)
            .map(|db| db.nearest_location(&coord)),
        loaded_databases
            .ipv6(&database)
            .map(|db| db.nearest_location(&coord)),
    ) {
        // closest in both
        (Ok(v4_location), Ok(v6_location)) => {
            match coord
                .distance_2(&v4_location)
                .partial_cmp(&coord.distance_2(&v6_location))
            {
                // v4 closer
                Some(Ordering::Less) => Ok(v4_location),
                // v6 closer
                _ => Ok(v6_location),
            }
        }
        // ipv4 only
        (Ok(location), _) => Ok(location),
        // ipv6 only
        (_, Ok(location)) => Ok(location),
        // neither
        (_, _) => Err("No locations found in either database".to_string()),
    }
}

/// Associated City, State, and Country for a Coordinate
#[tauri::command]
pub fn location_info(
    loaded_databases: State<'_, GlobalDatabases>,
    database: DatabaseQuery,
    coord: Coordinate,
) -> Result<Option<LocationInfo>, String> {
    match (
        loaded_databases
            .ipv4(&database)
            .map(|db| db.get_location_info(&coord)),
        loaded_databases
            .ipv6(&database)
            .map(|db| db.get_location_info(&coord)),
    ) {
        (Ok(Some(info)), _) | (_, Ok(Some(info))) => Ok(Some(info)),
        (Ok(None), _) | (_, Ok(None)) => Ok(None),
        (Err(_), Err(_)) => Err("No databases loaded".to_string()),
    }
}

/// Our coordinate based on the current database
#[tauri::command]
pub async fn my_location(
    loaded_databases: State<'_, GlobalDatabases>,
    ip: State<'_, PublicIpAddress>,
    database: DatabaseQuery,
) -> Result<Coordinate, String> {
    lookup_ip(loaded_databases, database, *ip)
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
