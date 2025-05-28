use std::{
    fs::File,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    sync::{Arc, RwLock},
    thread,
};

use dashmap::DashMap;
use ipgeo::{Coordinate, Database, GenericDatabase, Location};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State, ipc::Channel};
use tauri_specta::Event;

/// Load a IP-Geolocation database into the program from the filename.
#[tauri::command]
#[specta::specta]
pub fn load_database(
    app: AppHandle,
    state: State<'_, DbState>,
    path: PathBuf,
    err: Channel<String>,
) {
    if state.ipv4_db.db_exists(&path).is_some() || state.ipv6_db.db_exists(&path).is_some() {
        return;
    }

    if state.loading_db.read().expect("read loading").is_some() {
        err.send("Database is already loading".to_string()).ok();
    }

    tracing::info!("loading database from {path:?}");

    state.loading_db.write().expect("write loading").replace(
        path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string()),
    );
    DbStateChange::emit(&app, &state);

    thread::spawn(move || {
        let state: State<'_, DbState> = app.try_state().unwrap();

        let file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                err.send(e.to_string()).ok();
                return;
            }
        };
        let db = match ipgeo::from_read(file) {
            Ok(db) => db,
            Err(e) => {
                err.send(e.to_string()).ok();
                return;
            }
        };

        let info = match db {
            GenericDatabase::Ipv4(db) => state.ipv4_db.insert(path, db),
            GenericDatabase::Ipv6(db) => state.ipv6_db.insert(path, db),
        };

        tracing::info!("finished loading database {:?}", info.name);

        state.loading_db.write().expect("write loading").take();
        DbStateChange::emit(&app, &state);
    });
}

/// Unload the database, freeing up memory.
#[tauri::command]
#[specta::specta]
pub async fn unload_database(
    app: AppHandle,
    state: State<'_, DbState>,
    db: DbInfo,
) -> Result<(), String> {
    tracing::info!("unloading database {:?}", db.name);

    state.ipv4_db.remove(&db);
    state.ipv6_db.remove(&db);

    DbStateChange::emit(&app, &state);

    Ok(())
}

/// Set the given database as the selected database for lookups.
#[tauri::command]
#[specta::specta]
pub fn set_selected_database(app: AppHandle, state: State<'_, DbState>, db: DbInfo) {
    tracing::info!("set selected database as {:?}", db.name);

    state.ipv4_db.set_selected(&db);
    state.ipv6_db.set_selected(&db);

    DbStateChange::emit(&app, &state);
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
    match ip {
        IpAddr::V4(ip) => state.ipv4_db.get(ip),
        IpAddr::V6(ip) => state.ipv6_db.get(ip),
    }
}

#[derive(Default)]
pub struct DbState {
    ipv4_db: DbCollection<Ipv4Addr>,
    ipv6_db: DbCollection<Ipv6Addr>,
    loading_db: RwLock<Option<String>>,
}

impl DbState {
    fn info(&self) -> DbStateInfo {
        DbStateInfo {
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

pub struct DbCollection<B> {
    pub selected: RwLock<Option<(DbInfo, Arc<Database<B>>)>>,
    pub loaded: DashMap<DbInfo, Arc<Database<B>>>,
}

impl<B> Default for DbCollection<B> {
    fn default() -> Self {
        Self {
            selected: RwLock::new(None),
            loaded: DashMap::new(),
        }
    }
}

impl<B> DbCollection<B>
where
    B: Ord + Clone,
    ipgeo::SteppedIp<B>: ipgeo::StepLite,
{
    pub fn insert(&self, path: PathBuf, db: Database<B>) -> DbInfo {
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

        let info = DbInfo { name, path };
        let db = Arc::new(db);

        self.loaded.insert(info.clone(), db.clone());
        self.selected
            .write()
            .expect("open selected")
            .replace((info.clone(), db));

        info
    }

    pub fn remove(&self, info: &DbInfo) {
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

    pub fn db_exists(&self, path: &PathBuf) -> Option<DbInfo> {
        self.loaded
            .iter()
            .filter(|kv| &kv.key().path == path)
            .map(|kv| kv.key().clone())
            .next()
    }

    pub fn info(&self) -> DbCollectionInfo {
        let selected = self
            .selected
            .read()
            .expect("read selected")
            .as_ref()
            .map(|(info, _)| info.clone());
        let loaded = self.loaded.iter().map(|kv| kv.key().clone()).collect();

        DbCollectionInfo { selected, loaded }
    }

    pub fn get(&self, ip: B) -> Option<LookupInfo> {
        self.selected
            .read()
            .expect("read selected")
            .as_ref()
            .and_then(|(_, db)| db.get(ip))
            .map(|(crd, loc)| LookupInfo { crd, loc })
    }

    pub fn set_selected(&self, info: &DbInfo) {
        if let Some(db) = self.loaded.get(info) {
            *self.selected.write().expect("open selected") = Some((info.clone(), db.clone()));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
pub struct DbStateInfo {
    pub ipv4: DbCollectionInfo,
    pub ipv6: DbCollectionInfo,
    pub loading: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
pub struct DbCollectionInfo {
    pub selected: Option<DbInfo>,
    pub loaded: Vec<DbInfo>,
}

/// Fired any time the state of loaded or selected databases are changed on the backend.
#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct DbStateChange(DbStateInfo);

impl DbStateChange {
    pub fn emit(app: &AppHandle, state: &DbState) {
        let _ = Self(state.info()).emit(app);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
pub struct DbInfo {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct LookupInfo {
    pub crd: Coordinate,
    pub loc: Location,
}
