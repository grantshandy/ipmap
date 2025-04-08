use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use dashmap::DashMap;
use ipgeo::{Coordinate, Database, Location};
use serde::{Deserialize, Serialize};
use specta::Type;

pub struct DatabaseState<B> {
    pub selected: Arc<RwLock<Option<(DatabaseInfo, Arc<Database<B>>)>>>,
    pub loaded: DashMap<DatabaseInfo, Arc<Database<B>>>,
}

impl<B> Default for DatabaseState<B> {
    fn default() -> Self {
        Self {
            selected: Arc::new(RwLock::new(None)),
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
        tracing::info!("unloading database {:?}", info.name);

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
            .filter_map(|kv| {
                if &kv.key().path == path {
                    Some(kv.key().clone())
                } else {
                    None
                }
            })
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
