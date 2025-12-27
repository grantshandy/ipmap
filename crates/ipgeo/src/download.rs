#[cfg(feature = "download")]
use std::borrow::Cow;
use std::{
    collections::HashMap,
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
use compact_str::CompactString;
use dashmap::DashMap;
use futures::StreamExt;
use indexmap::IndexSet;
use rustc_hash::FxBuildHasher;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::io::StreamReader;
use unix_time::Instant;

const REPORT_GAP: Duration = Duration::from_secs(1);

#[cfg(feature = "download")]
#[derive(Debug)]
pub struct CombinedDatabaseSource<'a> {
    pub ipv4_csv_url: Cow<'a, str>,
    pub ipv6_csv_url: Cow<'a, str>,
    pub is_num: bool,
}

impl<Ip: GenericIp> SingleDatabase<Ip> {
    #[cfg(feature = "download")]
    pub async fn download(
        csv_url: impl AsRef<str>,
        is_num: bool,
        cb: impl Fn(usize, usize) + Send + Sync + 'static,
    ) -> anyhow::Result<Self> {
        let resp = reqwest::get(csv_url.as_ref()).await?;

        let content_length = resp.content_length();

        let mut last_reported = Instant::now();
        let mut count = 0;

        let mut cb = |v: usize| {
            count += v;

            if let Some(content_length) = content_length
                && last_reported.elapsed() >= REPORT_GAP
            {
                cb(count, content_length as usize);
                last_reported = Instant::now();
            }
        };

        let stream = resp
            .bytes_stream()
            .map(|item| item.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
            .map(|item| {
                item.map(|i| {
                    cb(i.len());
                    i
                })
            });
        let stream_reader = StreamReader::new(stream);

        // 2. Wrap it in the GzipDecoder
        // This handles all the "deflate" logic and header checking for you
        let mut gz_decoder = BufReader::new(GzipDecoder::new(stream_reader));

        let mut ips = IpLookupTable::new();
        let mut locations = LocationStore::default();

        let mut record = csv::ByteRecord::new();
        let mut line = Vec::new();

        let ip_parser = if is_num {
            Ip::from_num_bytes
        } else {
            Ip::from_str_bytes
        };

        while gz_decoder.read_until(b'\n', &mut line).await? > 0 {
            if csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(&line[..])
                .read_byte_record(&mut record)
                .is_ok_and(|v| v && record.len() >= NUM_RECORDS)
            {
                crate::reader::csv::read_record(&record, ip_parser, &mut ips, &mut locations)?;
            }

            // Important: clear the scratch buffer for the next line
            line.clear();
        }

        Ok(Self { ips, locations })
    }
}

impl CombinedDatabase {
    #[cfg(feature = "download")]
    pub async fn download<'a>(
        source: CombinedDatabaseSource<'a>,
        cb: impl Fn(usize, usize) + Send + Sync + 'static,
    ) -> anyhow::Result<Self> {
        let last = Arc::new((AtomicU64::new(0), AtomicU32::new(0)));
        let state = Arc::new((AtomicUsize::new(0), AtomicUsize::new(0)));

        let cb = Arc::new(cb);

        // Clone Arcs for the first closure
        let last_clone = last.clone();
        let state_clone = state.clone();
        let cb_clone = cb.clone();

        let add_val = move |v: usize| {
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
        let add_total = move |v: usize| {
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

        let locations = Arc::try_unwrap(locations)
            .map_err(|_| anyhow::anyhow!("Failed to unwrap locations Arc"))?
            .into_store();

        Ok(CombinedDatabase {
            ipv4: ipv4??,
            ipv6: ipv6??,
            locations,
        })
    }
}

async fn concurrent_table_download<Ip: GenericIp>(
    url: String,
    is_num: bool,
    locations: Arc<ConcurrentLocationStore>,
    len_report: impl Fn(usize),
    chunk_report: impl Fn(usize),
) -> anyhow::Result<IpLookupTable<Ip, Coordinate>> {
    let resp = reqwest::get(url).await?;
    if let Some(cl) = resp.content_length() {
        len_report(cl as usize);
    }

    // 1. Convert the Stream into an AsyncRead
    let stream = resp
        .bytes_stream()
        .map(|item| item.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
        .map(|item| {
            item.map(|i| {
                chunk_report(i.len());
                i
            })
        });
    let stream_reader = StreamReader::new(stream);

    // 2. Wrap it in the GzipDecoder
    // This handles all the "deflate" logic and header checking for you
    let mut gz_decoder = BufReader::new(GzipDecoder::new(stream_reader));

    let mut table = IpLookupTable::new();
    let mut record = csv::ByteRecord::new();
    let mut line = Vec::new();

    let ip_parser = if is_num {
        Ip::from_num_bytes
    } else {
        Ip::from_str_bytes
    };

    while gz_decoder.read_until(b'\n', &mut line).await? > 0 {
        if csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(&line[..])
            .read_byte_record(&mut record)
            .is_ok_and(|v| v && record.len() >= NUM_RECORDS)
        {
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
        }

        // Important: clear the scratch buffer for the next line
        line.clear();
    }

    Ok(table)
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
    /// Insert a new location into the store using &self (concurrently).
    fn insert(
        &self,
        coord: Coordinate,
        create_location: &dyn Fn(&ConcurrentStringDict) -> Result<LocationIndices, Error>,
    ) -> Result<(), Error> {
        // 1. Check if coordinate exists (Fast path)
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
        let strings = self.strings.into_string_dict();

        let mut loc_vec: Vec<(usize, LocationIndices)> = self.loc_storage.into_iter().collect();
        loc_vec.sort_unstable_by_key(|(k, _)| *k);

        let locations: IndexSet<LocationIndices, FxBuildHasher> =
            loc_vec.into_iter().map(|(_, v)| v).collect();

        let coordinates: HashMap<Coordinate, LocationKey, FxBuildHasher> =
            self.coordinates.into_iter().collect();

        LocationStore {
            coordinates,
            locations,
            strings,
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

        // Note: Optimization possible here to avoid allocation if key exists,
        // but requires checking lookup with a reference which DashMap supports.
        // For strict correctness with ownership:
        let s = CompactString::from_utf8(item).ok()?.to_lowercase();

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
