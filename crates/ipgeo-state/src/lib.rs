use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use dashmap::DashMap;
use ipgeo::{Coordinate, Database, Location};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

const DNS_LOOKUP_TIMEOUT: Duration = Duration::from_millis(300);

pub mod commands;
mod my_loc;

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
