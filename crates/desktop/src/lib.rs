use std::{
    fs::File,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    sync::RwLock,
};

use ipgeo::{Coordinate, GenericDatabase, Location};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_specta::{Builder, Event, collect_commands, collect_events};

mod db_state;

use db_state::{DatabaseInfo, DatabaseState, DatabaseStateInfo};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        .events(collect_events![DatabaseStateChange])
        .commands(collect_commands![
            load_database,
            unload_database,
            database_state,
            set_selected_database,
            lookup_ip
        ]);

    #[cfg(debug_assertions)]
    builder
        .export(specta_typescript::Typescript::default(), "../../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
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
    ipv4_db: DatabaseState<Ipv4Addr>,
    ipv6_db: DatabaseState<Ipv6Addr>,
    loading_db: RwLock<Option<String>>,
}

impl AppState {
    fn info(&self) -> AppStateInfo {
        AppStateInfo {
            ipv4: self.ipv4_db.info(),
            ipv6: self.ipv6_db.info(),
            loading: self
                .loading_db
                .read()
                .map(|s| s.clone())
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
pub struct AppStateInfo {
    pub ipv4: DatabaseStateInfo,
    pub ipv6: DatabaseStateInfo,
    pub loading: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
struct DatabaseStateChange(AppStateInfo);

impl DatabaseStateChange {
    pub fn emit(app: &AppHandle, state: &AppState) {
        let _ = Self(state.info()).emit(app);
    }
}

#[tauri::command]
#[specta::specta]
async fn load_database(
    app: AppHandle,
    state: State<'_, AppState>,
    path: PathBuf,
) -> Result<DatabaseInfo, String> {
    if let Some(exists) = state.ipv4_db.db_exists(&path) {
        return Ok(exists);
    }

    if let Some(exists) = state.ipv6_db.db_exists(&path) {
        return Ok(exists);
    }

    if state.loading_db.read().expect("read loading").is_some() {
        return Err("Database is already loading".to_string());
    }

    tracing::info!("loading database from {path:?}");

    state.loading_db.write().expect("write loading").replace(
        path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string()),
    );
    DatabaseStateChange::emit(&app, &state);


    let file = File::open(&path).map_err(|e| e.to_string())?;
    let db = ipgeo::from_read(file).map_err(|e| e.to_string())?;

    let info = match db {
        GenericDatabase::Ipv4(db) => state.ipv4_db.insert(path, db),
        GenericDatabase::Ipv6(db) => state.ipv6_db.insert(path, db),
    };

    tracing::info!("finished loading database {:?}", info.name);

    state.loading_db.write().expect("write loading").take();
    DatabaseStateChange::emit(&app, &state);

    Ok(info)
}

#[tauri::command]
#[specta::specta]
async fn unload_database(app: AppHandle, state: State<'_, AppState>, db: DatabaseInfo) -> Result<(), String> {
    state.ipv4_db.remove(&db);
    state.ipv6_db.remove(&db);

    DatabaseStateChange::emit(&app, &state);

    Ok(())
}

#[tauri::command]
#[specta::specta]
fn set_selected_database(app: AppHandle, state: State<'_, AppState>, db: DatabaseInfo) {
    state.ipv4_db.set_selected(&db);
    state.ipv6_db.set_selected(&db);

    DatabaseStateChange::emit(&app, &state);
}

#[tauri::command]
#[specta::specta]
fn database_state(state: State<'_, AppState>) -> AppStateInfo {
    state.info()
}

#[tauri::command]
#[specta::specta]
fn lookup_ip(state: State<'_, AppState>, ip: IpAddr) -> Option<(Coordinate, Location)> {
    match ip {
        IpAddr::V4(ip) => state.ipv4_db.get(ip),
        IpAddr::V6(ip) => state.ipv6_db.get(ip),
    }
}
