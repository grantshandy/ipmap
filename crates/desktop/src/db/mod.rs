use std::{
    collections::HashSet,
    fmt, fs,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

use dashmap::{DashMap, Entry};
use ipgeo::{CombinedDatabase, Coordinate, Database, GenericDatabase, Location, SingleDatabase};
use rkyv::rancor;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

pub mod commands;
pub mod my_loc;

pub use ipgeo::LookupInfo;

const DNS_LOOKUP_TIMEOUT: Duration = Duration::from_millis(300);
const DB_EXTENSION: &str = "ipgeodb";

pub struct DbState {
    ipv4_db: DbSet<SingleDatabase<Ipv4Addr>>,
    ipv6_db: DbSet<SingleDatabase<Ipv6Addr>>,
    combined: DbSet<CombinedDatabase>,
    cache_dir: PathBuf,
}

impl DbState {
    pub fn new(handle: &AppHandle) -> Result<Self, tauri::Error> {
        Ok(DbState {
            ipv4_db: DbSet::default(),
            ipv6_db: DbSet::default(),
            combined: DbSet::default(),
            cache_dir: handle.path().app_data_dir()?.join("dbs"),
        })
    }

    fn info(&self) -> DbStateInfo {
        DbStateInfo {
            ipv4: self.ipv4_db.info(),
            ipv6: self.ipv6_db.info(),
            combined: self.combined.info(),
        }
    }

    pub fn emit_info(&self, app: &AppHandle) {
        let _ = DbStateChange(self.info()).emit(app);
    }

    fn loaded(&self) -> HashSet<String> {
        self.combined
            .loaded()
            .chain(self.ipv4_db.loaded())
            .chain(self.ipv6_db.loaded())
            .collect()
    }

    pub async fn insert(&self, source: &DatabaseSource, db: DynamicDatabase) -> anyhow::Result<()> {
        let path = self
            .cache_dir
            .join(source.to_filename())
            .with_extension(DB_EXTENSION);

        let db = tokio::task::spawn_blocking(move || {
            db.write_to_file(&path)?;
            anyhow::Result::<DynamicDatabase>::Ok(db)
        })
        .await??;

        let name = source.to_string();

        match db {
            DynamicDatabase::Combined(db) => self.combined.insert(name, db),
            DynamicDatabase::Generic(GenericDatabase::Ipv4(db)) => self.ipv4_db.insert(name, db),
            DynamicDatabase::Generic(GenericDatabase::Ipv6(db)) => self.ipv6_db.insert(name, db),
        }

        Ok(())
    }

    pub async fn refresh_cache(&self) -> anyhow::Result<()> {
        tracing::info!("refreshing from cache dir {:?}", self.cache_dir);

        let loaded = self.loaded();
        let cache_dir = self.cache_dir.clone();

        let dbs = tokio::task::spawn_blocking(move || {
            fs::create_dir_all(&cache_dir)?;

            let databases = fs::read_dir(&cache_dir)?
                .filter_map(|d| d.ok())
                .filter(|d| {
                    d.path().ends_with(DB_EXTENSION) && d.file_type().is_ok_and(|ft| ft.is_file())
                })
                .map(|d| d.path());

            let mut resp = Vec::new();

            for path in databases {
                tracing::debug!("found {path:?}");

                let Some(name) = path.file_stem().and_then(|fs| fs.to_str().to_owned()) else {
                    tracing::warn!("{path:?} has no valid file stem, skipping");
                    continue;
                };

                if loaded.contains(name) {
                    tracing::warn!("{name} already loaded in the database, skipping");
                    continue;
                }

                match DynamicDatabase::read_from_path(&path) {
                    Ok(db) => resp.push((name.to_string(), db)),
                    Err(err) => {
                        tracing::error!("Failed to read {path:?}, skipping: {err}");
                        continue;
                    }
                }
            }

            anyhow::Result::<Vec<(String, DynamicDatabase)>>::Ok(resp)
        })
        .await??;

        for (name, db) in dbs {
            match db {
                DynamicDatabase::Combined(db) => self.combined.insert(name, db),
                DynamicDatabase::Generic(GenericDatabase::Ipv4(db)) => {
                    self.ipv4_db.insert(name, db)
                }
                DynamicDatabase::Generic(GenericDatabase::Ipv6(db)) => {
                    self.ipv6_db.insert(name, db)
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct DbStateInfo {
    pub ipv4: DbSetInfo,
    pub ipv6: DbSetInfo,
    pub combined: DbSetInfo,
}

impl Database<IpAddr> for DbState {
    fn get(&self, ip: IpAddr) -> Option<LookupInfo> {
        match ip {
            IpAddr::V4(ip) => self.ipv4_db.get(ip),
            IpAddr::V6(ip) => self.ipv6_db.get(ip),
        }
        .or_else(|| self.combined.get(ip))
    }

    fn get_coordinate(&self, ip: IpAddr) -> Option<Coordinate> {
        match ip {
            IpAddr::V4(ip) => self.ipv4_db.get_coordinate(ip),
            IpAddr::V6(ip) => self.ipv6_db.get_coordinate(ip),
        }
        .or_else(|| self.combined.get_coordinate(ip))
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.ipv4_db
            .get_location(crd)
            .or_else(|| self.ipv6_db.get_location(crd))
            .or_else(|| self.combined.get_location(crd))
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

    #[allow(dead_code)]
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

    pub fn loaded(&self) -> impl Iterator<Item = String> {
        self.loaded.iter().map(|kv| kv.key().clone())
    }
}

// TODO: move all selected call into single helper methods
impl<Ip, C> Database<Ip> for DbSet<C>
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

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum DynamicDatabase {
    Combined(CombinedDatabase),
    Generic(GenericDatabase),
}

impl DynamicDatabase {
    pub fn write_to_file(&self, path: &Path) -> anyhow::Result<()> {
        // TODO: optimize, compress!
        tracing::debug!("writing DynamicDatabase to {path:?}");
        fs::write(path, rkyv::to_bytes::<rancor::Error>(self)?)?;
        Ok(())
    }

    pub fn read_from_path(path: &Path) -> anyhow::Result<Self> {
        // TODO: optimize, decompress!
        tracing::debug!("reading DynamicDatabase from {path:?}");
        Ok(rkyv::from_bytes::<Self, rancor::Error>(&fs::read(path)?)?)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseSource {
    DbIpCombined,
    Geolite2Combined,
    SingleCsvGz {
        is_ipv6: bool,
        url: String,
        is_num: bool,
    },
    CombinedCsvGz {
        ipv4: String,
        ipv6: String,
        is_num: bool,
    },
    File(PathBuf),
}

fn url_filename_guess<'a>(path: &'a str) -> &'a str {
    path.rsplit_once("/")
        .map(|(_, name)| name)
        .unwrap_or("unknown")
}

impl DatabaseSource {
    pub fn to_filename(&self) -> String {
        match self {
            DatabaseSource::DbIpCombined => "dbip".into(),
            DatabaseSource::Geolite2Combined => "geolite2".into(),
            DatabaseSource::SingleCsvGz { url, .. } => url_filename_guess(&url).to_string(),
            DatabaseSource::CombinedCsvGz { ipv4, ipv6, .. } => {
                format!(
                    "{}-{}",
                    url_filename_guess(&ipv4),
                    url_filename_guess(&ipv6)
                )
            }
            DatabaseSource::File(p) => p
                .file_stem()
                .and_then(|fs| fs.to_str())
                .unwrap_or("unknown")
                .to_string(),
        }
    }
}

impl fmt::Display for DatabaseSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseSource::DbIpCombined => f.write_str("DB-IP City"),
            DatabaseSource::Geolite2Combined => f.write_str("Geolite2 City"),
            DatabaseSource::SingleCsvGz { url, .. } => f.write_str(url_filename_guess(url)),
            DatabaseSource::CombinedCsvGz { ipv4, ipv6, .. } => {
                f.write_str(url_filename_guess(&ipv4))?;
                f.write_str("/")?;
                f.write_str(url_filename_guess(&ipv6))
            }
            DatabaseSource::File(path) => f.write_str(
                path.file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown File"),
            ),
        }
    }
}
