use std::{
    fs,
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use dashmap::{DashMap, DashSet};
use ipgeo::{ArchivedGenericDatabase, Coordinate, Database, Location};

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

use crate::{
    archive::{self, FileResource},
    disk::{ArchivedDynamicDatabase, DatabaseSource, DiskArchive, DynamicDatabase},
};

/// Tracks the state of all loaded IP geolocation databases, including IPv4, IPv6,
/// and combined IPv4/IPv6 archives.
///
/// [`DbState`] manages the DB cache directory, loaded databases, and selection state.
/// It provides methods for inserting new databases, removing or selecting them,
/// refreshing the cache from disk, and emitting state change events to the frontend.
pub struct DbState {
    cache_dir: PathBuf,
    ipv4: DbSet<Ipv4Addr>,
    ipv6: DbSet<Ipv6Addr>,
    combined: DbSet<IpAddr>,
    loaded_checksums: DashSet<u64>,
}

impl DbState {
    /// Constructs a new [`DbState`] using the application's data directory.
    pub fn new<R: Runtime>(handle: &AppHandle<R>) -> Result<Self, tauri::Error> {
        Ok(DbState {
            cache_dir: handle.path().app_data_dir()?.join("dbs"),
            ipv4: DbSet::default(),
            ipv6: DbSet::default(),
            combined: DbSet::default(),
            loaded_checksums: DashSet::default(),
        })
    }

    /// Returns a summary of the current database state,
    /// including loaded and selected databases, intended to be sent to the frontend.
    pub fn info(&self) -> DbStateInfo {
        DbStateInfo {
            ipv4: self.ipv4.info(),
            ipv6: self.ipv6.info(),
            combined: self.combined.info(),
        }
    }

    /// Emits a state change event to the frontend, reflecting the current database state.
    pub fn emit_info<R: Runtime>(&self, app: &AppHandle<R>) {
        let _ = DbStateChange(self.info()).emit(app);
    }

    /// Inserts a new database archive into the cache and updates the loaded/selected state.
    ///
    /// The database is serialized, checksummed, and memory-mapped before being added.
    pub async fn insert(&self, source: DatabaseSource, db: DynamicDatabase) -> anyhow::Result<()> {
        let cache_dir = self.cache_dir.clone();

        let fa = tokio::task::spawn_blocking(move || {
            FileResource::create(&cache_dir, &DiskArchive { source, db })
        })
        .await??;

        if self.loaded_checksums.contains(&fa.checksum()) {
            tracing::warn!("'{}' already loaded, skipping", &fa.source);
            return Ok(());
        }

        self.loaded_checksums.insert(fa.checksum());

        match &fa.db {
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

    /// Removes a database from all sets.
    pub fn remove(&self, source: &DatabaseSource) {
        self.combined.remove(source);
        self.ipv4.remove(source);
        self.ipv6.remove(source);
    }

    /// Sets the selected database for all sets if available.
    pub fn set_selected(&self, source: &DatabaseSource) {
        self.combined.set_selected(source);
        self.ipv4.set_selected(source);
        self.ipv6.set_selected(source);
    }

    /// Loads any new archives from the cache directory, updating the loaded state.
    ///
    /// Skips databases that are already loaded, and logs errors for any corrupt or unreadable archives.
    pub async fn refresh_cache(&self) -> anyhow::Result<()> {
        tracing::debug!("refreshing from cache dir {:?}", self.cache_dir);

        let loaded_checksums = self.loaded_checksums.clone();
        let cache_dir = self.cache_dir.clone();

        let dbs = tokio::task::spawn_blocking(move || {
            fs::create_dir_all(&cache_dir)?;

            let res = archive::resource_dir_list(&cache_dir)?
                .filter(|(_, c)| !loaded_checksums.contains(&c))
                .filter_map(|(path, _)| match FileResource::open(&path) {
                    Ok(db) => {
                        tracing::debug!("loaded {path:?}");
                        Some(db)
                    }
                    Err(err) => {
                        tracing::error!("failed to read {path:?}, skipping: {err}");
                        None
                    }
                })
                .collect::<Vec<_>>();

            anyhow::Result::<Vec<FileResource<DiskArchive>>>::Ok(res)
        })
        .await??;

        for archive in dbs {
            self.loaded_checksums.insert(archive.checksum());

            match &archive.db {
                ArchivedDynamicDatabase::Combined(_) => self.combined.insert(archive),
                ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv4(_)) => {
                    self.ipv4.insert(archive)
                }
                ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv6(_)) => {
                    self.ipv6.insert(archive)
                }
            }
        }

        Ok(())
    }
}

impl Database<IpAddr> for DbState {
    fn get_coordinate(&self, ip: IpAddr) -> Option<Coordinate> {
        match ip {
            IpAddr::V4(ip) => self.ipv4.get_coordinate(ip),
            IpAddr::V6(ip) => self.ipv6.get_coordinate(ip),
        }
        .or_else(|| self.combined.get_coordinate(ip))
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.combined
            .get_location(crd)
            .or_else(|| self.ipv4.get_location(crd))
            .or_else(|| self.ipv6.get_location(crd))
    }
}

/// Summary of the loaded and selected databases for each IP type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct DbStateInfo {
    pub ipv4: DbSetInfo,
    pub ipv6: DbSetInfo,
    pub combined: DbSetInfo,
}

/// Information about the loaded and selected databases in a [`DbSet`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct DbSetInfo {
    pub selected: Option<DatabaseSource>,
    pub loaded: Vec<DatabaseSource>,
}

/// Manages a set of loaded database archives for a specific IP type (IPv4, IPv6, or combined).
///
/// [`DbSet`] tracks loaded databases, the currently selected database, and provides
/// methods for insertion, removal, selection, and querying.
pub struct DbSet<C> {
    selected: RwLock<Option<Arc<FileResource<DiskArchive>>>>,
    loaded: DashMap<DatabaseSource, Arc<FileResource<DiskArchive>>>,
    _marker: PhantomData<C>,
}

impl<C> Default for DbSet<C> {
    /// Creates an empty [`DbSet`] with no loaded or selected databases.
    fn default() -> Self {
        Self {
            selected: RwLock::new(None),
            loaded: DashMap::new(),
            _marker: PhantomData,
        }
    }
}

impl<C> DbSet<C> {
    /// Inserts a new database archive, making it the selected database.
    pub fn insert(&self, db: FileResource<DiskArchive>) {
        let db = Arc::new(db);

        self.loaded
            .insert(DatabaseSource::from(&db.source), db.clone());
        self.selected.write().expect("open selected").replace(db);
    }

    /// Removes and deletes a database archive by source,
    /// updating the selected database if necessary.
    pub fn remove(&self, name: &DatabaseSource) {
        let mut selected = self.selected.write().expect("open selected");

        let selected_is_name = selected
            .as_ref()
            .is_some_and(|sel_db| &sel_db.source == name);

        let Some((_, fa)) = self.loaded.remove(name) else {
            return;
        };

        if selected_is_name {
            *selected = self.loaded.iter().map(|kv| kv.value().clone()).next();
        }

        match Arc::into_inner(fa).map(|fa| fa.delete()) {
            Some(Err(err)) => tracing::error!("failed to delete {name}: {err}"),
            None => tracing::error!("failed to remove {name}, other references."),
            _ => tracing::info!("Successfully removed database: {name}"),
        }
    }

    /// Returns true if no databases are loaded in this set.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.loaded.is_empty()
    }

    /// Returns true if a database with the given source exists in this set.
    #[allow(dead_code)]
    pub fn exists(&self, name: DatabaseSource) -> bool {
        self.loaded.contains_key(&name)
    }

    /// Returns information about the loaded and selected databases in this set.
    pub fn info(&self) -> DbSetInfo {
        DbSetInfo {
            selected: self
                .selected
                .read()
                .expect("read selected")
                .as_ref()
                .map(|s| DatabaseSource::from(&s.source)),
            loaded: self
                .loaded
                .iter()
                .map(|kv| DatabaseSource::from(&kv.value().source))
                .collect(),
        }
    }

    /// Sets the selected database by source, if it exists.
    pub fn set_selected(&self, name: &DatabaseSource) {
        if let Some(kv) = self.loaded.get(name) {
            *self.selected.write().expect("open selected") = Some(kv.value().clone());
        }
    }

    /// Executes a function on the selected database, if any.
    fn on_selected<T>(&self, f: impl Fn(&ArchivedDynamicDatabase) -> Option<T>) -> Option<T> {
        self.selected
            .read()
            .expect("read selected")
            .as_ref()
            .and_then(|ar| f(&ar.db))
    }
}

impl<C> Database<C> for DbSet<C>
where
    C: Copy,
    ArchivedDynamicDatabase: Database<C>,
{
    fn get_coordinate(&self, ip: C) -> Option<Coordinate> {
        self.on_selected(|db| db.get_coordinate(ip))
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.on_selected(|db| Database::<C>::get_location(db, crd))
    }
}

/// Event fired whenever the state of loaded or selected databases changes.
///
/// Used to notify the frontend of updates to the database state.
#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct DbStateChange(DbStateInfo);
