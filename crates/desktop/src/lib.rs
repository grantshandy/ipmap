use std::{fs::File, path::PathBuf, net::IpAddr};

use dashmap::DashMap;
use ipgeo::{Coordinate, GeoDatabase, Location};
use serde::{Deserialize, Serialize};
use specta::Type;
use specta_typescript::Typescript;
use tauri::{AppHandle, Manager, State};
use tauri_specta::{Builder, Event, collect_commands, collect_events};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        .events(collect_events![UpdateDatabases])
        .commands(collect_commands![load_database, list_databases, lookup_ip]);

    #[cfg(debug_assertions)]
    builder
        .export(Typescript::default(), "../../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            app.manage(AppState::default());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Default)]
struct AppState {
    databases: DashMap<DatabaseInfo, GeoDatabase>,
}

#[tauri::command]
#[specta::specta]
async fn load_database(
    app: AppHandle,
    state: State<'_, AppState>,
    path: PathBuf,
) -> Result<DatabaseInfo, String> {
    // TODO:
    // if let Some(kv) = state.databases.get(&path) {
    //     println!("already exists!");
    //     return Ok(kv.value().1.clone());
    // }

    println!("loading...");
    let file = File::open(&path).map_err(|e| e.to_string())?;
    let db = GeoDatabase::from_read(file).map_err(|e| e.to_string())?;
    println!("loaded");

    let info = DatabaseInfo::new(path.clone(), &db);
    state.databases.insert(info.clone(), db);

    UpdateDatabases::from_state(&state)
        .emit(&app)
        .map_err(|e| e.to_string())?;

    Ok(info)
}

#[tauri::command]
#[specta::specta]
fn list_databases(state: State<'_, AppState>) -> UpdateDatabases {
    UpdateDatabases::from_state(&state)
}

#[tauri::command]
#[specta::specta]
fn lookup_ip(state: State<'_, AppState>, db: DatabaseInfo, ip: IpAddr) -> Result<Option<(Coordinate, Location)>, String> {
    state
        .databases
        .get(&db)
        .ok_or(format!("no database found for {db:?}"))
        .map(|db| db.value().get(ip))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
struct DatabaseInfo {
    path: PathBuf,
    ipv6: bool,
}

impl DatabaseInfo {
    pub fn new(path: PathBuf, db: &GeoDatabase) -> Self {
        Self {
            path,
            ipv6: db.is_ipv6(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct UpdateDatabases(Vec<DatabaseInfo>);

impl UpdateDatabases {
    fn from_state(state: &State<'_, AppState>) -> Self {
        Self(
            state
                .databases
                .iter()
                .map(|kv| kv.key().clone())
                .collect(),
        )
    }
}
