use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

use dashmap::{DashMap, Entry};
use ipgeo::{
    CombinedDatabase, Coordinate, Database, GenericDatabase, GenericIp, Location, SingleDatabase,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

pub mod commands;
pub mod my_loc;

pub use ipgeo::LookupInfo;

const DNS_LOOKUP_TIMEOUT: Duration = Duration::from_millis(300);

pub struct DbState {
    ipv4_db: DbSet<SingleDatabase<Ipv4Addr>>,
    ipv6_db: DbSet<SingleDatabase<Ipv6Addr>>,
    combined: DbSet<CombinedDatabase>,
    cache_dir: PathBuf,
    use_combined: bool,
}

impl DbState {
    pub fn new(handle: &AppHandle) -> Result<Self, tauri::Error> {
        Ok(DbState {
            ipv4_db: DbSet::default(),
            ipv6_db: DbSet::default(),
            combined: DbSet::default(),
            cache_dir: handle.path().app_data_dir()?.join("dbs"),
            use_combined: false,
        })
    }

    fn info(&self) -> DbStateInfo {
        DbStateInfo {
            ipv4: self.ipv4_db.info(),
            ipv6: self.ipv6_db.info(),
        }
    }

    pub fn emit_info(&self, app: &AppHandle) {
        let _ = DbStateChange(self.info()).emit(app);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct DbStateInfo {
    pub ipv4: DbSetInfo,
    pub ipv6: DbSetInfo,
}

impl Database<IpAddr> for DbState {
    fn get(&self, ip: IpAddr) -> Option<LookupInfo> {
        match ip {
            IpAddr::V4(ip) => self.ipv4_db.get(ip),
            IpAddr::V6(ip) => self.ipv6_db.get(ip),
        }
    }

    fn get_coordinate(&self, ip: IpAddr) -> Option<Coordinate> {
        match ip {
            IpAddr::V4(ip) => self.ipv4_db.get_coordinate(ip),
            IpAddr::V6(ip) => self.ipv6_db.get_coordinate(ip),
        }
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.ipv4_db
            .get_location(crd)
            .or_else(|| self.ipv6_db.get_location(crd))
    }
}

struct DbSet<C> {
    pub selected: RwLock<Option<(String, Arc<C>)>>,
    pub loaded: DashMap<String, Arc<C>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct DbSetInfo {
    pub selected: Option<String>,
    pub loaded: Vec<String>,
}

impl<C> Default for DbSet<C> {
    fn default() -> Self {
        Self {
            selected: RwLock::new(None),
            loaded: DashMap::new(),
        }
    }
}

impl<C> DbSet<C> {
    pub fn insert(&self, name: impl AsRef<str>, db: C) {
        self.insert_arc(name, Arc::new(db));
    }

    pub fn insert_arc(&self, name: impl AsRef<str>, db: Arc<C>) {
        self.loaded.insert(name.as_ref().to_string(), db.clone());
        self.selected
            .write()
            .expect("open selected")
            .replace((name.as_ref().to_string(), db));
    }

    pub fn remove(&self, name: impl AsRef<str>) {
        if let Entry::Occupied(kv) = self.loaded.entry(name.as_ref().to_string()) {
            kv.remove();
        }

        // set the selected as the next database, if it exists
        let mut selected = self.selected.write().expect("open selected");
        if selected.as_ref().is_some_and(|(path, _)| path == path) {
            *selected = self
                .loaded
                .iter()
                .map(|kv| {
                    let (a, b) = kv.pair();
                    (a.clone(), b.clone())
                })
                .next();
        }
    }

    pub fn exists(&self, name: impl Into<String>) -> bool {
        self.loaded.contains_key(&name.into())
    }

    pub fn info(&self) -> DbSetInfo {
        let selected = self
            .selected
            .read()
            .expect("read selected")
            .as_ref()
            .map(|(path, _)| path)
            .cloned();

        let loaded: Vec<String> = self.loaded.iter().map(|kv| kv.key().clone()).collect();

        DbSetInfo { selected, loaded }
    }

    pub fn set_selected(&self, name: impl Into<String>) {
        if let Some(kv) = self.loaded.get(&name.into()) {
            let (path, db) = kv.pair();
            *self.selected.write().expect("open selected") = Some((path.clone(), db.clone()));
        }
    }
}

// TODO: move all selected call into single helper methods
impl<Ip: GenericIp, C> Database<Ip> for DbSet<C>
where
    C: Database<Ip>,
{
    fn get(&self, ip: Ip) -> Option<LookupInfo> {
        self.selected
            .read()
            .expect("read selected")
            .as_ref()
            .and_then(|(_, db)| db.get(ip))
    }

    fn get_coordinate(&self, ip: Ip) -> Option<Coordinate> {
        self.selected
            .read()
            .expect("read selected")
            .as_ref()
            .and_then(|(_, db)| db.get_coordinate(ip))
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.selected
            .read()
            .expect("read selected")
            .as_ref()
            .and_then(|(_, db)| db.get_location(crd))
    }
}

/// Fired any time the state of loaded or selected databases are changed on the backend.
#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct DbStateChange(DbStateInfo);

#[derive(Serialize, Deserialize)]
enum DynamicDatabase {
    Combined(CombinedDatabase),
    Generic(GenericDatabase),
}
