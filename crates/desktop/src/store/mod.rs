use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use compact_str::{CompactString, ToCompactString};
use dashmap::DashMap;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use futures::StreamExt;
use ipgeo::{
    CombinedDatabase, Coordinate, Database, GenericIp, IpLookupTable, Location,
    location::LocationStore,
};

pub mod commands;
pub mod sources;

use serde::{Deserialize, Serialize};
use sources::DatabaseSource;
use specta::Type;
use tauri::{App, AppHandle, Manager};
use tauri_specta::Event;
use time::UtcDateTime;

const EXTENSION: &str = "ipgeo";
const REPORT_PERCENTAGE: u64 = 5;

pub struct DatabaseStore {
    handle: AppHandle,
    data_dir: PathBuf,
    loaded: DashMap<usize, NamedDatabase>,
    selected: AtomicUsize,
    cache_initialized: AtomicBool,
    loading: AtomicBool,
}

impl DatabaseStore {
    pub fn new(app: &App) -> Result<Self, tauri::Error> {
        let data_dir = app.path().app_local_data_dir()?.join("databases");

        tracing::debug!("caching databases at {data_dir:?}");

        Ok(Self {
            handle: app.handle().clone(),
            data_dir,
            loaded: DashMap::new(),
            selected: 0.into(),
            cache_initialized: false.into(),
            loading: false.into(),
        })
    }

    /// Read the previously downloaded ip-location databases from the disk.
    ///
    /// TODO:
    pub fn init_cache(&self) -> anyhow::Result<()> {
        if self.cache_initialized.load(Ordering::Relaxed) {
            tracing::debug!("not initializing cache");
            return Ok(());
        }

        tracing::debug!("initializing cache");

        self.loading.store(true, Ordering::SeqCst);
        self.emit_update();

        // TODO: make concurrent async stream?
        fs::read_dir(&self.data_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension() == Some(OsStr::new(EXTENSION)))
            .map(NamedDatabase::deserialize_from_path)
            .filter_map(|r| match r {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::error!("Error loading internal database: {e}");
                    None
                }
            })
            .enumerate()
            .for_each(|(key, value)| {
                self.loaded.insert(key, value);
            });

        self.cache_initialized.store(true, Ordering::Relaxed);
        self.loading.store(false, Ordering::SeqCst);
        self.emit_update();

        tracing::debug!("finished initializing store");

        Ok(())
    }

    pub async fn download(
        &self,
        source: DatabaseSource,
        progress: impl Fn(f32),
        stage: impl Fn(&'static str),
    ) -> anyhow::Result<()> {
        progress(0.0);

        let dl_prog = |p| progress(p * 0.666);

        stage("Downloading IPv4 Database");

        let locations = LocationStore::default();
        let (ipv4, locations) =
            csv_read_wrapper::<Ipv4Addr>(source.ipv4_urls, locations, |p| dl_prog(p * 0.5)).await?;

        stage("Downloading IPv6 Database");

        let (ipv6, locations) =
            csv_read_wrapper::<Ipv6Addr>(source.ipv6_urls, locations, |p| dl_prog(0.5 + p * 0.5))
                .await?;

        let path = self
            .data_dir
            .join(source.metadata.file_name.as_str())
            .with_extension(EXTENSION);

        let disk = NamedDatabase {
            db: CombinedDatabase::new(ipv4, ipv6, locations),
            metadata: source.metadata,
            created: UtcDateTime::now(),
        };

        stage("Compressing");

        disk.serialize_to_path(path, |p| progress(0.666 + p * 0.333))?;

        progress(1.0);

        self.insert(disk);

        Ok(())
    }

    pub fn insert(&self, disk: NamedDatabase) {
        let key = self.loaded.len();
        self.loaded.insert(key, disk);
        self.selected.store(key, Ordering::Relaxed);
    }

    fn on_selected<R>(&self, cb: impl Fn(&CombinedDatabase) -> R) -> Option<R> {
        self.loaded
            .get(&self.selected.load(Ordering::Relaxed))
            .map(|disk| cb(&disk.db))
    }

    pub fn emit_update(&self) {
        let info = self.info();

        tracing::debug!("{info:#?}");

        let _ = <DatabaseStoreInfo as Event>::emit(&info, &self.handle);
    }

    pub fn info(&self) -> DatabaseStoreInfo {
        let loaded = self
            .loaded
            .iter()
            .map(|kv| kv.value().metadata.display_name.to_string())
            .collect();

        let selected = self
            .loaded
            .get(&self.selected.load(Ordering::Relaxed))
            .map(|kv| kv.value().metadata.display_name.to_string());

        DatabaseStoreInfo {
            loaded,
            selected,
            loading: self.loading.load(Ordering::SeqCst),
        }
    }
}

impl Database<IpAddr> for DatabaseStore {
    fn get_coordinate(&self, ip: IpAddr) -> Option<Coordinate> {
        self.on_selected(|db| db.get_coordinate(ip)).flatten()
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.on_selected(|db| db.get_location(crd)).flatten()
    }
}

async fn csv_read_wrapper<Ip: GenericIp + Send + 'static>(
    urls: &[&str],
    mut locations: LocationStore,
    progress: impl Fn(f32),
) -> anyhow::Result<(IpLookupTable<Ip, Coordinate>, LocationStore)> {
    let (reader, mut writer) = pipe::pipe();

    let parse_task = tokio::task::spawn_blocking::<
        _,
        anyhow::Result<(IpLookupTable<Ip, Coordinate>, LocationStore)>,
    >(move || {
        let mut ips = IpLookupTable::new();

        ipgeo::reader::csv::read(GzDecoder::new(reader), true, &mut ips, &mut locations)?;

        Ok((ips, locations))
    });

    let mut last_error: Option<reqwest::Error> = None;

    for url in urls {
        let (content_length, mut stream) = match reqwest::get(*url).await {
            Ok(resp) => (resp.content_length(), resp.bytes_stream()),
            Err(err) => {
                tracing::error!("Failed to download database: {err}");
                last_error = Some(err);
                continue;
            }
        };

        if content_length.is_none() {
            progress(0.0);
        }

        let report_percentage = content_length.unwrap_or_default() / (100 / REPORT_PERCENTAGE);
        let mut acc: u64 = 0;
        let mut last_percentage: u64 = 0;

        while let Some(bytes) = stream.next().await {
            let bytes = bytes?;

            if let Some(content_length) = content_length {
                acc += bytes.len() as u64;

                if acc >= last_percentage + report_percentage {
                    last_percentage = acc;
                    progress((acc as f64 / content_length as f64) as f32)
                }
            }

            // doesn't block, writes to buffered csv reader
            writer.write_all(&bytes)?;
        }

        progress(1.0);

        break;
    }

    if let Some(error) = last_error {
        return Err(anyhow::anyhow!(error));
    }

    let parse_task = parse_task.await??;

    Ok(parse_task)
}

#[derive(Serialize, Deserialize)]
pub struct NamedDatabase {
    db: CombinedDatabase,
    metadata: NamedDatabaseMetadata,
    created: UtcDateTime,
}

impl NamedDatabase {
    pub fn from_file(path: PathBuf) -> Result<Self, ipgeo::Error> {
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_compact_string();

        let copyright = if file_name.contains("geolite2") {
            sources::GEOLITE2_CITY.metadata.copyright.clone()
        } else if file_name.contains("dbip") {
            sources::DBIP_CITY.metadata.copyright.clone()
        } else {
            "Unknown".into()
        };

        Ok(Self {
            db: ipgeo::detect(&path)?.into(),
            metadata: NamedDatabaseMetadata {
                display_name: file_name.clone(),
                file_name,
                copyright,
            },
            created: UtcDateTime::now(),
        })
    }

    pub fn serialize_to_path(&self, path: PathBuf, progress: impl Fn(f32)) -> anyhow::Result<()> {
        if path.exists() {
            fs::remove_file(&path)?;
        }

        postcard::to_io(
            &self,
            GzEncoder::new(
                ProgressWriter::new(File::create(&path)?, 53_000_000, progress),
                Compression::fast(), // TODO: switch to best?
            ),
        )?;
        Ok(())
    }

    pub fn deserialize_from_path(path: PathBuf) -> anyhow::Result<Self> {
        let mut file = GzDecoder::new(File::open(path)?);

        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let me = postcard::from_bytes(&buf)?;
        Ok(me)
    }
}

#[derive(Serialize, Deserialize)]
pub struct NamedDatabaseMetadata {
    display_name: CompactString,
    file_name: CompactString,
    copyright: CompactString,
}

#[derive(Clone, Type, Event, Debug, Serialize, Deserialize)]
pub struct DatabaseStoreInfo {
    loaded: Vec<String>,
    selected: Option<String>,
    loading: bool,
}

struct ProgressWriter<W: io::Write, F: Fn(f32)> {
    inner: W,
    total_size: u64,
    bytes_written: u64,
    callback: F,
}

impl<W: io::Write, F: Fn(f32)> ProgressWriter<W, F> {
    fn new(inner: W, total_size: u64, callback: F) -> Self {
        Self {
            inner,
            total_size,
            bytes_written: 0,
            callback,
        }
    }
}

impl<W: io::Write, F: Fn(f32)> io::Write for ProgressWriter<W, F> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(buf)?;
        self.bytes_written += written as u64;

        let progress = if self.total_size == 0 {
            1.0
        } else {
            (self.bytes_written as f32) / (self.total_size as f32)
        };

        (self.callback)(progress.min(1.0));

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
