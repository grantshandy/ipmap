use std::{fs::File, net::Ipv4Addr, path::PathBuf};

use tauri::State;

use crate::{geoip::database::Database, DatabaseState};

use self::database::Info;

pub mod database {
    include!(concat!(env!("OUT_DIR"), "/database.rs"));
}

#[tauri::command]
pub async fn lookup_ip(
    databases: State<'_, DatabaseState>,
    database: Option<PathBuf>,
    ip: Ipv4Addr,
) -> Result<Option<database::Location>, String> {
    let res = match database {
        Some(path) => databases
            .get(&path)
            .ok_or("database not found".to_string())?
            .value()
            .db
            .get(&u32::from(ip))
            .cloned(),
        None => database::DATABASE
            .as_ref()
            .ok_or("no internal database set")?
            .db
            .get(&u32::from(ip))
            .cloned(),
    };

    Ok(res)
}

#[tauri::command]
pub async fn load_database(
    databases: State<'_, DatabaseState>,
    path: Option<PathBuf>,
) -> Result<Option<database::Info>, String> {
    let Some(path) = path else {
        lazy_static::initialize(&database::DATABASE);

        if database::DATABASE.is_none() {
            tracing::warn!("no internal database set");
        }

        return Ok(database::DATABASE.as_ref().map(|db| db.info.clone()));
    };

    tracing::info!("reading db at {path:?}");

    let db_file = File::open(&path).map_err(|e| e.to_string())?;
    let (db, locations) = database::read_csv(&db_file).map_err(|e| e.to_string())?;

    let info = Info {
        name: path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        path: Some(path.to_string_lossy().to_string()),
        build_time: database::build_time(),
        attribution_text: None,
        locations
    };

    databases.insert(
        path,
        Database {
            db,
            info: info.clone(),
        },
    );

    Ok(Some(info))
}

#[tauri::command]
pub fn list_databases(databases: State<'_, DatabaseState>) -> Vec<Info> {
    let mut databases: Vec<Info> = databases.iter().map(|v| v.info.clone()).collect();

    if let Some(internal) = database::DATABASE.as_ref() {
        databases.push(internal.info.clone());
    }

    databases
}
