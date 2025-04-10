use std::{
    fs::File,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use dashmap::DashMap;
use ipgeo::{Coordinate, Database, GenericDatabase, Location};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State};
use tauri_specta::Event;

/// Load a IP-Geolocation database into the program from the filename.
#[tauri::command]
#[specta::specta]
pub async fn load_database(
    app: AppHandle,
    state: State<'_, GlobalDatabaseState>,
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

/// Unload the database, freeing up memory.
#[tauri::command]
#[specta::specta]
pub async fn unload_database(
    app: AppHandle,
    state: State<'_, GlobalDatabaseState>,
    db: DatabaseInfo,
) -> Result<(), String> {
    tracing::info!("unloading database {:?}", db.name);

    state.ipv4_db.remove(&db);
    state.ipv6_db.remove(&db);

    DatabaseStateChange::emit(&app, &state);

    Ok(())
}

/// Set the given database as the selected database for lookups.
#[tauri::command]
#[specta::specta]
pub fn set_selected_database(
    app: AppHandle,
    state: State<'_, GlobalDatabaseState>,
    db: DatabaseInfo,
) {
    tracing::info!("set selected database as {:?}", db.name);

    state.ipv4_db.set_selected(&db);
    state.ipv6_db.set_selected(&db);

    DatabaseStateChange::emit(&app, &state);
}

/// Retrieve the current state of the database.
/// This info is given out in [`DatabaseStateChange`], but this is useful for getting it at page load, for example.
#[tauri::command]
#[specta::specta]
pub fn database_state(state: State<'_, GlobalDatabaseState>) -> GlobalDatabaseStateInfo {
    state.info()
}

/// Lookup a given IP address in the currently selected database(s).
#[tauri::command]
#[specta::specta]
pub fn lookup_ip(
    state: State<'_, GlobalDatabaseState>,
    ip: IpAddr,
) -> Option<(Coordinate, Location)> {
    match ip {
        IpAddr::V4(ip) => state.ipv4_db.get(ip),
        IpAddr::V6(ip) => state.ipv6_db.get(ip),
    }
}

/// Fired any time the state of loaded or selected databases are changed on the backend.
#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct DatabaseStateChange(GlobalDatabaseStateInfo);

impl DatabaseStateChange {
    pub fn emit(app: &AppHandle, state: &GlobalDatabaseState) {
        let _ = Self(state.info()).emit(app);
    }
}

#[derive(Default)]
pub struct GlobalDatabaseState {
    ipv4_db: DatabaseState<Ipv4Addr>,
    ipv6_db: DatabaseState<Ipv6Addr>,
    loading_db: RwLock<Option<String>>,
}

impl GlobalDatabaseState {
    fn info(&self) -> GlobalDatabaseStateInfo {
        GlobalDatabaseStateInfo {
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
pub struct GlobalDatabaseStateInfo {
    pub ipv4: DatabaseStateInfo,
    pub ipv6: DatabaseStateInfo,
    pub loading: Option<String>,
}

pub struct DatabaseState<B> {
    pub selected: RwLock<Option<(DatabaseInfo, Arc<Database<B>>)>>,
    pub loaded: DashMap<DatabaseInfo, Arc<Database<B>>>,
}

impl<B> Default for DatabaseState<B> {
    fn default() -> Self {
        Self {
            selected: RwLock::new(None),
            loaded: DashMap::new(),
        }
    }
}

impl<B> DatabaseState<B>
where
    B: Ord + Clone,
    ipgeo::SteppedIp<B>: ipgeo::StepLite,
{
    pub fn insert(&self, path: PathBuf, db: Database<B>) -> DatabaseInfo {
        let mut name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_owned();

        let duplicates = self
            .loaded
            .iter()
            .filter(|kv| &kv.key().name == &name)
            .count();

        if duplicates > 0 {
            name.push_str(&format!(" ({duplicates})"));
        }

        let info = DatabaseInfo { name, path };
        let db = Arc::new(db);

        self.loaded.insert(info.clone(), db.clone());
        self.selected
            .write()
            .expect("open selected")
            .replace((info.clone(), db));

        info
    }

    pub fn remove(&self, info: &DatabaseInfo) {
        self.loaded.remove(info);

        let mut selected = self.selected.write().expect("open selected");

        if selected.as_ref().is_some_and(|(loaded, _)| loaded == info) {
            *selected = self
                .loaded
                .iter()
                .map(|kv| (kv.key().clone(), kv.value().clone()))
                .next();
        }
    }

    pub fn db_exists(&self, path: &PathBuf) -> Option<DatabaseInfo> {
        self.loaded
            .iter()
            .filter(|kv| &kv.key().path == path)
            .map(|kv| kv.key().clone())
            .next()
    }

    pub fn info(&self) -> DatabaseStateInfo {
        let selected = self
            .selected
            .read()
            .expect("read selected")
            .as_ref()
            .map(|(info, _)| info.clone());
        let loaded = self.loaded.iter().map(|kv| kv.key().clone()).collect();

        DatabaseStateInfo { selected, loaded }
    }

    pub fn get(&self, ip: B) -> Option<(Coordinate, Location)> {
        self.selected
            .read()
            .expect("read selected")
            .as_ref()
            .and_then(|(_, db)| db.get(ip))
    }

    pub fn set_selected(&self, info: &DatabaseInfo) {
        if let Some(db) = self.loaded.get(info) {
            *self.selected.write().expect("open selected") = Some((info.clone(), db.clone()));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
pub struct DatabaseStateInfo {
    pub selected: Option<DatabaseInfo>,
    pub loaded: Vec<DatabaseInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
pub struct DatabaseInfo {
    pub name: String,
    pub path: PathBuf,
}
