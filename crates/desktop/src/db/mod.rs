use std::{
    collections::HashSet,
    fmt, fs,
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use dashmap::DashMap;
use ipgeo::{
    ArchivedGenericDatabase, CombinedDatabase, Coordinate, Database, GenericDatabase, Location,
};

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

pub mod archive;
pub mod commands;
pub mod my_loc;

pub use ipgeo::LookupInfo;
use time::UtcDateTime;

use crate::db::archive::{DiskArchive, FileArchive};

const DNS_LOOKUP_TIMEOUT: Duration = Duration::from_millis(300);
const DB_EXTENSION: &str = "ipgeodb";

pub struct DbState {
    cache_dir: PathBuf,
    ipv4: DbSet<Ipv4Addr>,
    ipv6: DbSet<Ipv6Addr>,
    combined: DbSet<IpAddr>,
}

impl DbState {
    pub fn new(handle: &AppHandle) -> Result<Self, tauri::Error> {
        Ok(DbState {
            cache_dir: handle.path().app_data_dir()?.join("dbs"),
            ipv4: DbSet::default(),
            ipv6: DbSet::default(),
            combined: DbSet::default(),
        })
    }

    fn info(&self) -> DbStateInfo {
        DbStateInfo {
            ipv4: self.ipv4.info(),
            ipv6: self.ipv6.info(),
            combined: self.combined.info(),
        }
    }

    pub fn emit_info(&self, app: &AppHandle) {
        let _ = DbStateChange(self.info()).emit(app);
    }

    pub async fn insert(&self, source: DatabaseSource, db: DynamicDatabase) -> anyhow::Result<()> {
        let path = self
            .cache_dir
            .join(generate_db_timestamp(&source))
            .with_extension(DB_EXTENSION);

        let fa = tokio::task::spawn_blocking(move || {
            FileArchive::create(&path, &DiskArchive { source, db })
        })
        .await??;

        match &fa.get_data().db {
            ArchivedDynamicDatabase::Combined(_) => self.combined.insert(fa),
            ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv4(_)) => {
                self.ipv4.insert(fa)
            }
            ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv6(_)) => {
                self.ipv6.insert(fa)
            }
        }

        Ok(())
    }

    pub fn remove(&self, source: &DatabaseSource) {
        self.combined.remove(source);
        self.ipv4.remove(source);
        self.ipv6.remove(source);
    }

    pub fn set_selected(&self, source: &DatabaseSource) {
        self.combined.set_selected(source);
        self.ipv4.set_selected(source);
        self.ipv6.set_selected(source);
    }

    pub async fn refresh_cache(&self) -> anyhow::Result<()> {
        tracing::info!("refreshing from cache dir {:?}", self.cache_dir);

        let mut loaded = self
            .combined
            .loaded()
            .chain(self.ipv4.loaded())
            .chain(self.ipv6.loaded())
            .collect::<HashSet<DatabaseSource>>();
        let cache_dir = self.cache_dir.clone();

        let dbs = tokio::task::spawn_blocking(move || {
            fs::create_dir_all(&cache_dir)?;

            let databases = fs::read_dir(&cache_dir)?
                .filter_map(|d| d.ok())
                .filter(|d| {
                    let ok = d.path().extension().is_some_and(|ext| ext == DB_EXTENSION)
                        && d.file_type().is_ok_and(|ft| ft.is_file());

                    tracing::debug!(
                        "dir entry {} {:?}",
                        (if ok { "OK" } else { "NOT OK" }),
                        d.path()
                    );

                    ok
                })
                .map(|d| d.path());

            let mut resp = Vec::new();

            for path in databases {
                match FileArchive::open(&path) {
                    Ok(db) => {
                        let ds = DatabaseSource::from(&db.get_data().source);

                        if loaded.contains(&ds) {
                            tracing::debug!("skipping {path:?}, already loaded");
                            continue;
                        }

                        loaded.insert(ds);
                        resp.push(db);

                        tracing::debug!("loaded {path:?}");
                    }
                    Err(err) => {
                        tracing::error!("failed to read {path:?}, skipping: {err}");
                    }
                }
            }

            anyhow::Result::<Vec<FileArchive>>::Ok(resp)
        })
        .await??;

        for db in dbs {
            match db.get_data().db {
                ArchivedDynamicDatabase::Combined(_) => self.combined.insert(db),
                ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv4(_)) => {
                    self.ipv4.insert(db)
                }
                ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv6(_)) => {
                    self.ipv6.insert(db)
                }
            }
        }

        Ok(())
    }
}

impl Database<IpAddr> for DbState {
    fn get_coordinate(&self, ip: IpAddr) -> Option<Coordinate> {
        if !self.combined.is_empty() {
            return self.combined.get_coordinate(ip);
        }

        match ip {
            IpAddr::V4(ip) => self.ipv4.get_coordinate(ip),
            IpAddr::V6(ip) => self.ipv6.get_coordinate(ip),
        }
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.combined
            .get_location(crd)
            .or_else(|| self.ipv4.get_location(crd))
            .or_else(|| self.ipv6.get_location(crd))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct DbStateInfo {
    pub ipv4: DbSetInfo,
    pub ipv6: DbSetInfo,
    pub combined: DbSetInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct DbSetInfo {
    pub selected: Option<DatabaseSource>,
    pub loaded: Vec<DatabaseSource>,
}

struct DbSet<C> {
    selected: RwLock<Option<Arc<FileArchive>>>,
    loaded: DashMap<DatabaseSource, Arc<FileArchive>>,
    _marker: PhantomData<C>,
}

impl<C> Default for DbSet<C> {
    fn default() -> Self {
        Self {
            selected: RwLock::new(None),
            loaded: DashMap::new(),
            _marker: PhantomData,
        }
    }
}

impl<C> DbSet<C> {
    pub fn insert(&self, db: FileArchive) {
        let db = Arc::new(db);

        self.loaded
            .insert(DatabaseSource::from(&db.get_data().source), db.clone());
        self.selected.write().expect("open selected").replace(db);
    }

    pub fn remove(&self, name: &DatabaseSource) {
        let mut selected = self.selected.write().expect("open selected");

        let selected_is_name = selected
            .as_ref()
            .is_some_and(|sel_db| &sel_db.get_data().source == name);

        let Some((_, fa)) = self.loaded.remove(name) else {
            return;
        };

        if selected_is_name {
            *selected = self.loaded.iter().map(|kv| kv.value().clone()).next();
        }

        match Arc::into_inner(fa).map(|fa| fa.delete()) {
            Some(Err(err)) => tracing::error!("failed to delete {name}: {err}"),
            None => tracing::error!("failed to remove {name}, other references."),
            _ => (),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.loaded.is_empty()
    }

    #[allow(dead_code)]
    pub fn exists(&self, name: DatabaseSource) -> bool {
        self.loaded.contains_key(&name)
    }

    pub fn info(&self) -> DbSetInfo {
        DbSetInfo {
            selected: self
                .selected
                .read()
                .expect("read selected")
                .as_ref()
                .map(|s| DatabaseSource::from(&s.get_data().source)),
            loaded: self
                .loaded
                .iter()
                .map(|kv| DatabaseSource::from(&kv.value().get_data().source))
                .collect(),
        }
    }

    pub fn set_selected(&self, name: &DatabaseSource) {
        if let Some(kv) = self.loaded.get(name) {
            *self.selected.write().expect("open selected") = Some(kv.value().clone());
        }
    }

    fn on_selected<T>(&self, f: impl Fn(&ArchivedDynamicDatabase) -> Option<T>) -> Option<T> {
        self.selected
            .read()
            .expect("read selected")
            .as_ref()
            .and_then(|db| f(&db.get_data().db))
    }

    pub fn loaded(&self) -> impl Iterator<Item = DatabaseSource> {
        self.loaded
            .iter()
            .map(|kv| DatabaseSource::from(&kv.value().get_data().source))
    }
}

impl Database<Ipv4Addr> for DbSet<Ipv4Addr> {
    fn get_coordinate(&self, ip: Ipv4Addr) -> Option<Coordinate> {
        self.on_selected(|db| db.get_coordinate(ip))
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.on_selected(|db| Database::<Ipv4Addr>::get_location(db, crd))
    }
}

impl Database<Ipv6Addr> for DbSet<Ipv6Addr> {
    fn get_coordinate(&self, ip: Ipv6Addr) -> Option<Coordinate> {
        self.on_selected(|db| db.get_coordinate(ip))
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.on_selected(|db| Database::<Ipv6Addr>::get_location(db, crd))
    }
}

impl Database<IpAddr> for DbSet<IpAddr> {
    fn get_coordinate(&self, ip: IpAddr) -> Option<Coordinate> {
        self.on_selected(|db| db.get_coordinate(ip))
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.on_selected(|db| Database::<IpAddr>::get_location(db, crd))
    }
}

/// Fired any time the state of loaded or selected databases are changed on the backend.
#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct DbStateChange(DbStateInfo);

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Type,
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum DatabaseSource {
    DbIpCombined,
    Geolite2Combined,
    File(String),
}

fn url_filename_guess<'a>(path: &'a str) -> &'a str {
    path.rsplit_once(&['/', '\\'])
        .map(|(_, last)| last)
        .unwrap_or("unknown")
}

fn generate_db_timestamp(src: &DatabaseSource) -> String {
    let now = UtcDateTime::now();

    match src {
        DatabaseSource::DbIpCombined => format!("dbip-{}", now.unix_timestamp()),
        DatabaseSource::Geolite2Combined => format!("geolite2-{}", now.unix_timestamp()),
        DatabaseSource::File(_) => format!("custom-{}", now.unix_timestamp()),
    }
}

impl fmt::Display for DatabaseSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseSource::DbIpCombined => f.write_str("DB-IP City"),
            DatabaseSource::Geolite2Combined => f.write_str("Geolite2 City"),
            DatabaseSource::File(path) => f.write_str(url_filename_guess(&path)),
        }
    }
}

impl fmt::Display for ArchivedDatabaseSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArchivedDatabaseSource::DbIpCombined => f.write_str("DB-IP City"),
            ArchivedDatabaseSource::Geolite2Combined => f.write_str("Geolite2 City"),
            ArchivedDatabaseSource::File(path) => f.write_str(url_filename_guess(&path)),
        }
    }
}

impl PartialEq<DatabaseSource> for ArchivedDatabaseSource {
    fn eq(&self, other: &DatabaseSource) -> bool {
        match (self, other) {
            (ArchivedDatabaseSource::File(path), DatabaseSource::File(other_path)) => {
                path == other_path
            }
            (ArchivedDatabaseSource::DbIpCombined, DatabaseSource::DbIpCombined) => true,
            (ArchivedDatabaseSource::Geolite2Combined, DatabaseSource::Geolite2Combined) => true,
            _ => false,
        }
    }
}

impl From<&ArchivedDatabaseSource> for DatabaseSource {
    fn from(value: &ArchivedDatabaseSource) -> Self {
        match value {
            ArchivedDatabaseSource::DbIpCombined => DatabaseSource::DbIpCombined,
            ArchivedDatabaseSource::Geolite2Combined => DatabaseSource::Geolite2Combined,
            ArchivedDatabaseSource::File(path) => DatabaseSource::File(path.to_string()),
        }
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum DynamicDatabase {
    Combined(CombinedDatabase),
    Generic(GenericDatabase),
}

impl Database<Ipv4Addr> for ArchivedDynamicDatabase {
    fn get_coordinate(&self, ip: Ipv4Addr) -> Option<Coordinate> {
        match self {
            ArchivedDynamicDatabase::Combined(db) => db.get_coordinate(ip.into()),
            ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv4(db)) => {
                db.get_coordinate(ip)
            }
            _ => None,
        }
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        match self {
            ArchivedDynamicDatabase::Combined(db) => db.get_location(crd),
            ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv4(db)) => {
                db.get_location(crd)
            }
            _ => None,
        }
    }
}

impl Database<Ipv6Addr> for ArchivedDynamicDatabase {
    fn get_coordinate(&self, ip: Ipv6Addr) -> Option<Coordinate> {
        match self {
            ArchivedDynamicDatabase::Combined(db) => db.get_coordinate(ip.into()),
            ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv6(db)) => {
                db.get_coordinate(ip)
            }
            _ => None,
        }
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        match self {
            ArchivedDynamicDatabase::Combined(db) => db.get_location(crd),
            ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv6(db)) => {
                db.get_location(crd)
            }
            _ => None,
        }
    }
}

impl Database<IpAddr> for ArchivedDynamicDatabase {
    fn get_coordinate(&self, ip: IpAddr) -> Option<Coordinate> {
        match (self, ip) {
            (ArchivedDynamicDatabase::Combined(db), ip) => db.get_coordinate(ip),
            (
                ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv4(db)),
                IpAddr::V4(ip),
            ) => db.get_coordinate(ip),
            (
                ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv6(db)),
                IpAddr::V6(ip),
            ) => db.get_coordinate(ip),
            _ => None,
        }
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        match self {
            ArchivedDynamicDatabase::Combined(db) => db.get_location(crd),
            ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv4(db)) => {
                db.get_location(crd)
            }
            ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv6(db)) => {
                db.get_location(crd)
            }
        }
    }
}
