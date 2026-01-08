use std::{
    borrow::Cow,
    num::NonZero,
    sync::{
        Arc,
        atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};

use crate::{
    CombinedDatabase, Coordinate, Error, GenericIp, SingleDatabase,
    locations::{
        CountryCode, LocationIndices, LocationKey, LocationStore, StringDict, StringDictKey,
    },
    reader::csv::*,
    treebitmap::IpLookupTable,
};

use async_compression::tokio::bufread::GzipDecoder;
use bytesize::ByteSize;
use compact_str::CompactString;
use csv_async::AsyncReaderBuilder;
use dashmap::DashMap;
use futures::StreamExt;
use rustc_hash::FxBuildHasher;
use tokio_util::io::StreamReader;
use unix_time::Instant;

const REPORT_GAP: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub struct CombinedDatabaseSource<'a> {
    pub ipv4_csv_url: Cow<'a, str>,
    pub ipv6_csv_url: Cow<'a, str>,
    pub is_num: bool,
}

impl<Ip: GenericIp> SingleDatabase<Ip> {
    pub async fn download(
        csv_url: impl AsRef<str>,
        is_num: bool,
        progress_report: impl Fn(u64, u64) + Send + Sync + 'static,
    ) -> anyhow::Result<Self> {
        let ip_parser = if is_num {
            Ip::from_num_bytes
        } else {
            Ip::from_str_bytes
        };

        let start = std::time::Instant::now();

        let resp = reqwest::get(csv_url.as_ref()).await?.error_for_status()?;

        let content_length = resp.content_length();

        let mut last_reported = Instant::now();
        let mut count = 0;

        let mut cb = |v: u64| {
            count += v;

            if let Some(content_length) = content_length
                && last_reported.elapsed() >= REPORT_GAP
            {
                progress_report(count, content_length);
                last_reported = Instant::now();
            }
        };

        let stream = resp
            .bytes_stream()
            .map(|item| item.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
            .map(|item| {
                item.map(|i| {
                    cb(i.len() as u64);
                    i
                })
            });

        let mut reader = AsyncReaderBuilder::new()
            .has_headers(false)
            .buffer_capacity(64 * 1024) // 64KB internal buffer
            .create_reader(GzipDecoder::new(StreamReader::new(stream)));

        let mut byte_records = reader.byte_records();
        let mut size = 0;

        let mut ips = IpLookupTable::new();
        let mut locations = LocationStore::default();

        while let Some(res) = byte_records.next().await {
            let record = res?;

            if record.len() < NUM_RECORDS {
                return Err(anyhow::anyhow!("Not enough columns"));
            }

            let coord = coord_from_record(&record)?;

            locations.insert(coord, &|strings| {
                Ok(LocationIndices {
                    city: strings.insert_bytes(&record[CITY_IDX]),
                    region: strings.insert_bytes(&record[REGION_IDX]),
                    country_code: CountryCode::from(&record[COUNTRY_CODE_IDX]),
                })
            })?;

            for (addr, len) in Ip::range_subnets(
                ip_parser(&record[IP_RANGE_START_IDX])?,
                ip_parser(&record[IP_RANGE_END_IDX])?,
            ) {
                ips.insert(addr, len, coord);
            }

            size += record.as_slice().len() as u64;
        }

        let size_mb = ByteSize::b(size).as_mb();
        let elapsed = start.elapsed();

        tracing::debug!(
            "Downloaded {:.2} MB, decompressing/parsing at {:.2} MB/s in {} seconds",
            ByteSize::b(content_length.unwrap_or_default()).as_mb(),
            size_mb / elapsed.as_secs_f64(),
            elapsed.as_secs()
        );

        Ok(Self { ips, locations })
    }
}

impl CombinedDatabase {
    #[cfg(feature = "download")]
    pub async fn download<'a>(
        source: CombinedDatabaseSource<'a>,
        progress_report: impl Fn(u64, u64) + Send + Sync + 'static,
    ) -> anyhow::Result<Self> {
        let start = std::time::Instant::now();

        let last = Arc::new((AtomicU64::new(0), AtomicU32::new(0)));
        let state = Arc::new((AtomicU64::new(0), AtomicU64::new(0)));

        let cb = Arc::new(progress_report);

        // Clone Arcs for the first closure
        let last_clone = last.clone();
        let state_clone = state.clone();
        let cb_clone = cb.clone();

        let add_val = move |v: u64| {
            let (secs, nanos) = last_clone.as_ref();
            let (val, max) = state_clone.as_ref();

            val.fetch_add(v, Ordering::SeqCst);

            let now = Instant::now();
            let last = Instant::at(secs.load(Ordering::SeqCst), nanos.load(Ordering::SeqCst));

            if now.duration_since(last) >= REPORT_GAP {
                // update last
                secs.store(now.secs(), Ordering::SeqCst);
                nanos.store(now.subsec_nanos(), Ordering::SeqCst);

                cb_clone(val.load(Ordering::Relaxed), max.load(Ordering::Relaxed));
            }
        };

        // Clone Arc for second closure
        let state_clone2 = state.clone();
        let add_total = move |v: u64| {
            state_clone2.1.fetch_add(v, Ordering::SeqCst);
        };

        // Wrap locations in Arc for sharing
        let locations = Arc::new(ConcurrentLocationStore::default());

        let (ipv4, ipv6) = tokio::join!(
            tokio::spawn(concurrent_table_download(
                source.ipv4_csv_url.to_string(),
                source.is_num,
                locations.clone(),
                add_total.clone(),
                add_val.clone()
            )),
            tokio::spawn(concurrent_table_download(
                source.ipv6_csv_url.to_string(),
                source.is_num,
                locations.clone(),
                add_total,
                add_val,
            ))
        );

        let (ipv4, ipv4_len) = ipv4??;
        let (ipv6, ipv6_len) = ipv6??;

        let size_mb = ByteSize::b(ipv4_len + ipv6_len).as_mb();
        let elapsed = start.elapsed();

        tracing::debug!(
            "Downloaded {:.2} MB, decompressing/parsing at {:.2} MB/s in {} seconds",
            ByteSize::b(state.1.load(Ordering::SeqCst)).as_mb(),
            size_mb / elapsed.as_secs_f64(),
            elapsed.as_secs()
        );

        let locations = Arc::try_unwrap(locations)
            .map_err(|_| anyhow::anyhow!("Failed to unwrap locations Arc"))?
            .into_store();

        Ok(CombinedDatabase {
            ipv4,
            ipv6,
            locations,
        })
    }
}

async fn concurrent_table_download<Ip: GenericIp>(
    url: String,
    is_num: bool,
    locations: Arc<ConcurrentLocationStore>,
    len_report: impl Fn(u64) + Send + Sync,
    chunk_report: impl Fn(u64) + Send + Sync,
) -> anyhow::Result<(IpLookupTable<Ip, Coordinate>, u64)> {
    let ip_parser = if is_num {
        Ip::from_num_bytes
    } else {
        Ip::from_str_bytes
    };

    let resp = reqwest::get(url).await?.error_for_status()?;

    if let Some(cl) = resp.content_length() {
        len_report(cl);
    }

    let stream = resp
        .bytes_stream()
        .map(|item| item.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
        .map(|item| {
            item.map(|i| {
                chunk_report(i.len() as u64);
                i
            })
        });

    let mut reader = AsyncReaderBuilder::new()
        .has_headers(false)
        .buffer_capacity(64 * 1024) // 64KB internal buffer
        .create_reader(GzipDecoder::new(StreamReader::new(stream)));

    let mut byte_records = reader.byte_records();

    let mut table = IpLookupTable::new();
    let mut size = 0;

    while let Some(res) = byte_records.next().await {
        let record = res?;

        if record.len() < NUM_RECORDS {
            return Err(anyhow::anyhow!("Not enough columns"));
        }

        let coord = coord_from_record(&record)?;

        locations.insert(coord, &|strings| {
            Ok(LocationIndices {
                city: strings.insert_bytes(&record[CITY_IDX]),
                region: strings.insert_bytes(&record[REGION_IDX]),
                country_code: CountryCode::from(&record[COUNTRY_CODE_IDX]),
            })
        })?;

        for (addr, len) in Ip::range_subnets(
            ip_parser(&record[IP_RANGE_START_IDX])?,
            ip_parser(&record[IP_RANGE_END_IDX])?,
        ) {
            table.insert(addr, len, coord);
        }

        size += record.as_slice().len() as u64;
    }

    Ok((table, size))
}

/// A concurrent, thread-safe builder for LocationStore.
#[derive(Default)]
struct ConcurrentLocationStore {
    coordinates: DashMap<Coordinate, LocationKey, FxBuildHasher>,
    loc_lookup: DashMap<LocationIndices, LocationKey, FxBuildHasher>,
    loc_storage: DashMap<LocationKey, LocationIndices, FxBuildHasher>,
    loc_counter: AtomicUsize,
    strings: ConcurrentStringDict,
}

impl ConcurrentLocationStore {
    fn insert(
        &self,
        coord: Coordinate,
        create_location: &dyn Fn(&ConcurrentStringDict) -> Result<LocationIndices, Error>,
    ) -> Result<(), Error> {
        if self.coordinates.contains_key(&coord) {
            return Ok(());
        }

        if let dashmap::Entry::Vacant(entry) = self.coordinates.entry(coord) {
            let indices = create_location(&self.strings)?;

            let loc_key = *self.loc_lookup.entry(indices).or_insert_with(|| {
                let id = self.loc_counter.fetch_add(1, Ordering::Relaxed);
                self.loc_storage.insert(id, indices);
                id
            });

            entry.insert(loc_key);
        }

        Ok(())
    }

    /// Convert this concurrent structure back into the standard single-threaded LocationStore
    fn into_store(self) -> LocationStore {
        let mut loc_vec: Vec<(usize, LocationIndices)> = self.loc_storage.into_iter().collect();
        loc_vec.sort_unstable_by_key(|(k, _)| *k);

        LocationStore {
            coordinates: self.coordinates.into_iter().collect(),
            locations: loc_vec.into_iter().map(|(_, v)| v).collect(),
            strings: self.strings.into_string_dict(),
        }
    }
}

/// A concurrent database of strings.
pub struct ConcurrentStringDict {
    lookup: DashMap<CompactString, u32, FxBuildHasher>,
    storage: DashMap<u32, CompactString, FxBuildHasher>,
    counter: AtomicU32,
}

impl Default for ConcurrentStringDict {
    fn default() -> Self {
        Self {
            lookup: DashMap::with_hasher(FxBuildHasher::default()),
            storage: DashMap::with_hasher(FxBuildHasher::default()),
            counter: AtomicU32::new(1), // 1-based index (NonZero)
        }
    }
}

impl ConcurrentStringDict {
    pub fn insert_bytes(&self, item: &[u8]) -> Option<StringDictKey> {
        if item.is_empty() {
            return None;
        }

        let s_ref = std::str::from_utf8(item).ok()?;

        if let Some(idx) = self.lookup.get(s_ref) {
            return NonZero::new(*idx);
        }

        let s = CompactString::from(s_ref);

        let idx = *self.lookup.entry(s.clone()).or_insert_with(|| {
            let id = self.counter.fetch_add(1, Ordering::Relaxed);
            self.storage.insert(id, s);
            id
        });

        NonZero::new(idx)
    }

    /// Convert back to the original [`StringDict`]
    fn into_string_dict(self) -> StringDict {
        let mut vec: Vec<(u32, CompactString)> = self.storage.into_iter().collect();
        vec.sort_unstable_by_key(|(k, _)| *k);
        StringDict(vec.into_iter().map(|(_, v)| v).collect())
    }
}
