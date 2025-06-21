use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use dashmap::DashMap;
use ipgeo::{Coordinate, Database, Location};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

#[derive(Default)]
pub struct DbState {
    ipv4_db: DbCollection<Ipv4Addr>,
    ipv6_db: DbCollection<Ipv6Addr>,
    loading: RwLock<Option<PathBuf>>,
}

impl DbState {
    fn info(&self) -> DbStateInfo {
        DbStateInfo {
            ipv4: self.ipv4_db.info(),
            ipv6: self.ipv6_db.info(),
            loading: self.loading.read().map(|s| s.clone()).unwrap_or_default(),
        }
    }
}

pub struct DbCollection<B> {
    pub selected: RwLock<Option<(PathBuf, Arc<Database<B>>)>>,
    pub loaded: DashMap<PathBuf, Arc<Database<B>>>,
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
    pub fn insert(&self, path: &PathBuf, db: Database<B>) {
        let db = Arc::new(db);

        self.loaded.insert(path.clone(), db.clone());
        self.selected
            .write()
            .expect("open selected")
            .replace((path.clone(), db));
    }

    pub fn remove(&self, path: &PathBuf) {
        self.loaded.remove(path);

        let mut selected = self.selected.write().expect("open selected");

        if selected.as_ref().is_some_and(|(s, _)| s == path) {
            *selected = self
                .loaded
                .iter()
                .map(|kv| (kv.key().clone(), kv.value().clone()))
                .next();
        }
    }

    pub fn exists(&self, path: &PathBuf) -> bool {
        self.loaded.contains_key(path)
    }

    pub fn info(&self) -> DbCollectionInfo {
        let selected = self
            .selected
            .read()
            .expect("read selected")
            .as_ref()
            .map(|(path, _)| path.clone());

        let loaded: Vec<PathBuf> = self.loaded.iter().map(|kv| kv.key().clone()).collect();

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

    pub fn set_selected(&self, path: &PathBuf) {
        if let Some(db) = self.loaded.get(path) {
            *self.selected.write().expect("open selected") =
                Some((db.key().clone(), db.value().clone()));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct DbStateInfo {
    pub ipv4: DbCollectionInfo,
    pub ipv6: DbCollectionInfo,
    pub loading: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct DbCollectionInfo {
    pub loaded: Vec<PathBuf>,
    pub selected: Option<PathBuf>,
}

/// Fired any time the state of loaded or selected databases are changed on the backend.
#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct DbStateChange(DbStateInfo);

impl DbStateChange {
    pub fn emit(app: &AppHandle) {
        let state = app.state::<DbState>().inner();
        let _ = Self(state.info()).emit(app);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct LookupInfo {
    pub crd: Coordinate,
    pub loc: Location,
}

pub mod commands {
    use ipgeo::GenericDatabase;
    use std::{fs::File, net::IpAddr, path::PathBuf, thread};
    use tauri::{AppHandle, Manager, State, ipc::Channel};

    use super::{DbState, DbStateChange, DbStateInfo, LookupInfo};

    /// Load a IP-Geolocation database into the program from the filename.
    #[tauri::command]
    #[specta::specta]
    pub async fn load_database(
        app: AppHandle,
        state: State<'_, DbState>,
        path: PathBuf,
        err: Channel<String>,
    ) -> Result<(), String> {
        if state.ipv4_db.exists(&path) || state.ipv6_db.exists(&path) {
            err.send("Database with the same name already loaded".to_string())
                .ok();
        }

        if state.loading.read().expect("read loading").is_some() {
            err.send("Database is already loading".to_string()).ok();
        }

        tracing::info!("loading database from {path:?}");

        state
            .loading
            .write()
            .expect("write loading")
            .replace(path.clone());
        DbStateChange::emit(&app);

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

            match db {
                GenericDatabase::Ipv4(db) => state.ipv4_db.insert(&path, db),
                GenericDatabase::Ipv6(db) => state.ipv6_db.insert(&path, db),
            }

            tracing::info!("finished loading database {path:#?}");

            state.loading.write().expect("write loading").take();
            DbStateChange::emit(&app);
        });

        Ok(())
    }

    /// Unload the database, freeing up memory.
    #[tauri::command]
    #[specta::specta]
    pub async fn unload_database(
        app: AppHandle,
        state: State<'_, DbState>,
        path: PathBuf,
    ) -> Result<(), String> {
        tracing::info!("unloading database {path:#?}");

        state.ipv4_db.remove(&path);
        state.ipv6_db.remove(&path);

        DbStateChange::emit(&app);

        Ok(())
    }

    /// Set the given database as the selected database for lookups.
    #[tauri::command]
    #[specta::specta]
    pub fn set_selected_database(app: AppHandle, state: State<'_, DbState>, path: PathBuf) {
        tracing::info!("set selected database as {path:#?}");

        state.ipv4_db.set_selected(&path);
        state.ipv6_db.set_selected(&path);

        DbStateChange::emit(&app);
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

    /// Get a hostname with the system for a given IP address.
    #[tauri::command]
    #[specta::specta]
    pub fn lookup_dns(ip: IpAddr) -> Option<String> {
        dns_lookup::lookup_addr(&ip).ok()
    }

    /// Get a hostname with the system for a given IP address.
    #[tauri::command]
    #[specta::specta]
    pub fn lookup_host(host: &str) -> Option<IpAddr> {
        dns_lookup::lookup_host(host)
            .ok()
            .and_then(|i| i.first().copied())
    }
}
