use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

use dashmap::{DashMap, Entry};
use ipgeo::{Database, GenericIp};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

pub mod commands;
mod my_loc;

#[cfg(db_preloads)]
mod preloads;

pub use ipgeo::LookupInfo;

const DNS_LOOKUP_TIMEOUT: Duration = Duration::from_millis(300);

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

struct LoadedDb<Ip: GenericIp> {
    path: PathBuf,
    preloaded: bool,
    db: Arc<Database<Ip>>,
}

struct DbCollection<Ip: GenericIp> {
    pub selected: RwLock<Option<Arc<LoadedDb<Ip>>>>,
    pub loaded: DashMap<PathBuf, Arc<LoadedDb<Ip>>>,
}

impl<Ip: GenericIp> Default for DbCollection<Ip> {
    fn default() -> Self {
        Self {
            selected: RwLock::new(None),
            loaded: DashMap::new(),
        }
    }
}

impl<Ip: GenericIp> DbCollection<Ip> {
    pub fn insert(&self, path: &Path, db: Database<Ip>) {
        self.insert_arc(path, Arc::new(db), false);
    }

    pub fn insert_arc(&self, path: &Path, db: Arc<Database<Ip>>, preloaded: bool) {
        let loaded = Arc::new(LoadedDb {
            path: path.to_path_buf(),
            db,
            preloaded,
        });

        self.loaded.insert(path.to_path_buf(), loaded.clone());
        self.selected
            .write()
            .expect("open selected")
            .replace(loaded);
    }

    pub fn remove(&self, path: &PathBuf) {
        // remove the database if it isn't preloaded
        if let Entry::Occupied(kv) = self.loaded.entry(path.clone()) {
            if !kv.get().preloaded {
                kv.remove();
            } else {
                return;
            }
        }

        // set the selected as the next database, if it exists
        let mut selected = self.selected.write().expect("open selected");
        if selected.as_ref().is_some_and(|s| &s.path == path) {
            *selected = self.loaded.iter().map(|kv| kv.value().clone()).next();
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
            .map(|selected| selected.path.clone());

        let loaded: Vec<DbInfo> = self
            .loaded
            .iter()
            .map(|kv| DbInfo {
                path: kv.path.clone(),
                preloaded: kv.preloaded,
            })
            .collect();

        DbCollectionInfo { selected, loaded }
    }

    pub fn get(&self, ip: Ip) -> Option<LookupInfo> {
        self.selected
            .read()
            .expect("read selected")
            .as_ref()
            .and_then(|selected| selected.db.get(ip))
    }

    pub fn set_selected(&self, path: &PathBuf) {
        if let Some(db) = self.loaded.get(path) {
            *self.selected.write().expect("open selected") = Some(db.value().clone());
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
    pub loaded: Vec<DbInfo>,
    pub selected: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct DbInfo {
    path: PathBuf,
    preloaded: bool,
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
