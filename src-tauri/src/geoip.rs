//! Corresponding definitions in /frontend/src/bindings/index.ts

use std::{cmp::Ordering, net::IpAddr, path::PathBuf};

use ipdb_city::{Coordinate, Database, DatabaseInfo, IpRange, Ipv4Bytes, Ipv6Bytes, LocationInfo};
use tauri::State;

use crate::{DatabaseQuery, GlobalDatabases, PUBLIC_IP};

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

    let dbs = loaded_databases.get(&database);

    let coord = match ip {
        IpAddr::V4(ip) => dbs
            .ipv4
            .ok_or("no ipv4 database loaded".to_string())?
            .get(ip),
        IpAddr::V6(ip) => dbs
            .ipv6
            .ok_or("no ipv6 database loaded".to_string())?
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

    let dbs = loaded_databases.get(&database);

    let range = match ip {
        IpAddr::V4(ip) => dbs
            .ipv4
            .ok_or("no ipv4 database loaded".to_string())?
            .get_range(ip),
        IpAddr::V6(ip) => dbs
            .ipv6
            .ok_or("no ipv6 database loaded".to_string())?
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
    let dbs = loaded_databases.get(&database);
    let mut ranges = Vec::new();

    if let Some(db) = dbs.ipv4 {
        ranges.extend(db.get_ranges(&coord));
    }

    if let Some(db) = dbs.ipv6 {
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
    let dbs = loaded_databases.get(&database);

    match (
        dbs.ipv4.map(|db| db.nearest_location(&coord)),
        dbs.ipv6.map(|db| db.nearest_location(&coord)),
    ) {
        // closest in both
        (Some(v4_location), Some(v6_location)) => {
            match ipdb_city::coord_distance_2(&v4_location, &coord)
                .partial_cmp(&ipdb_city::coord_distance_2(&v6_location, &coord))
            {
                // v4 closer
                Some(Ordering::Less) => Ok(v4_location),
                // v6 closer
                _ => Ok(v6_location),
            }
        }
        // ipv4 only
        (Some(location), _) => Ok(location),
        // ipv6 only
        (_, Some(location)) => Ok(location),
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
    let dbs = loaded_databases.get(&database);

    match (
        dbs.ipv4.and_then(|db| db.get_location_info(&coord)),
        dbs.ipv6.and_then(|db| db.get_location_info(&coord)),
    ) {
        (Some(info), _) | (_, Some(info)) => Ok(Some(info)),
        (None, None) => Ok(None),
    }
}

/// Our coordinate based on the current database
#[tauri::command]
pub async fn my_location(
    loaded_databases: State<'_, GlobalDatabases>,
    database: DatabaseQuery,
) -> Result<Coordinate, String> {
    lookup_ip(loaded_databases, database, *PUBLIC_IP)
        .await
        .and_then(|loc| {
            loc.ok_or(format!(
                "no location found for your public ip address {}",
                *PUBLIC_IP
            ))
        })
}

/// Lookup the associated DNS address with a string.
#[tauri::command]
pub async fn dns_lookup_addr(ip: IpAddr) -> Option<String> {
    dns_lookup::lookup_addr(&ip).ok()
}

/// Lookup the associated IP address with a DNS address.
#[tauri::command]
pub async fn dns_lookup_host(host: String) -> Option<IpAddr> {
    dns_lookup::lookup_host(&host)
        .ok()
        .and_then(|ips| ips.get(0).copied())
}

/// Validate if a string is a global IP address.
#[tauri::command]
pub async fn validate_ip(ip: String) -> bool {
    ip.parse::<IpAddr>().is_ok_and(|ip| ip_rfc::global(&ip))
}
