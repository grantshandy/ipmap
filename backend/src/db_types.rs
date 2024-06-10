use std::collections::HashMap;
use std::fs::File;
use std::hash::{DefaultHasher, Hasher};
use std::net::Ipv4Addr;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use half::f16;
use rangemap::RangeInclusiveMap;
use time::OffsetDateTime;
use ts_rs::TS;

type Ipv4Bytes = u32;
type LocationKey = u32;
type StringKey = u32;

/// Acts like a HashMap for Ipv4 -> Location.
/// highly optimized for space in memory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Database {
    map: RangeInclusiveMap<Ipv4Bytes, LocationKey>,
    locations: HashMap<LocationKey, LocationEncoded>,
    string_dict: HashMap<StringKey, CompactString>,

    name: CompactString,
    attribution: Option<CompactString>,
    path: Option<PathBuf>,
    build_time: OffsetDateTime,
}

impl Database {
    pub fn from_csv(path: impl AsRef<Path>, attribution: Option<String>) -> eyre::Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)?;

        let file_stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let (name, path) = if attribution.is_some() {
            (format!("{file_stem} (built in)"), None)
        } else {
            (file_stem, Some(path.to_path_buf()))
        };

        let mut db = Self {
            map: RangeInclusiveMap::default(),
            locations: HashMap::new(),
            string_dict: HashMap::new(),

            name: CompactString::from(name),
            attribution: attribution.map(CompactString::from),
            path,
            build_time: OffsetDateTime::now_utc(),
        };

        for record in csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(file)
            .byte_records()
        {
            let record = record.expect("deserialize byte record");

            let (Some(ip_range_start), Some(ip_range_end)) = (
                str_from_byte_record(&record[0]).and_then(|r| r.parse::<u32>().ok()),
                str_from_byte_record(&record[1]).and_then(|r| r.parse::<u32>().ok()),
            ) else {
                return Err(eyre::eyre!("couldn't parse ip ranges"));
            };

            let (Some(latitude), Some(longitude)) = (
                str_from_byte_record(&record[7]).and_then(|s| s.parse::<f32>().ok()),
                str_from_byte_record(&record[8]).and_then(|s| s.parse::<f32>().ok()),
            ) else {
                continue;
            };

            db.insert(
                ip_range_start..=ip_range_end,
                Location {
                    latitude,
                    longitude,
                    city: str_from_byte_record(&record[5]),
                    country_code: str_from_byte_record(&record[2]),
                    state: str_from_byte_record(&record[3]),
                },
            );
        }

        Ok(db)
    }

    fn insert(&mut self, ip_range: RangeInclusive<Ipv4Bytes>, location: Location) {
        let latitude = f16::from_f32(location.latitude);
        let longitude = f16::from_f32(location.longitude);

        let loc_key = location_to_key(latitude, longitude);

        if !self.locations.contains_key(&loc_key) {
            let city = self.hash_and_insert_str(location.city);
            let country_code = self.hash_and_insert_str(location.country_code);
            let state = self.hash_and_insert_str(location.state);
    
            self.locations.insert(
                loc_key,
                LocationEncoded {
                    latitude,
                    longitude,
                    city,
                    country_code,
                    state,
                },
            );
        }

        self.map.insert(ip_range, loc_key);
    }

    fn hash_and_insert_str(&mut self, item: Option<String>) -> StringKey {
        item.map(|item| {
            let mut hasher = DefaultHasher::new();
            hasher.write(item.as_bytes());

            let mut key: u32 = hasher.finish() as u32;

            // size_of::<Option<T>>() > size_of::<T>(), so we store zero for a None value instead.
            // in the case that we come upon a key at zero (1 in 4 billion something idk) we catch that case.
            if key == 0 {
                key = 42; // magic random number.
            }

            if let Some(prev) = self.string_dict.get(&key) {
                if prev != &item {
                    tracing::warn!("strings \"{prev}\" and \"{item}\" collided");
                }
            }

            self.string_dict.insert(key, CompactString::from(item));

            key
        })
        .unwrap_or(0)
    }
}

#[allow(dead_code)]
impl Database {
    pub fn get(&self, ip: Ipv4Addr) -> Option<Location> {
        self.map
            .get(&u32::from(ip))
            .and_then(|k| self.locations.get(k))
            .map(|k| Location {
                latitude: k.latitude.to_f32(),
                longitude: k.longitude.to_f32(),
                city: self.str_from_dict(k.city),
                country_code: self.str_from_dict(k.country_code),
                state: self.str_from_dict(k.state),
            })
    }

    fn str_from_dict(&self, key: u32) -> Option<String> {
        self.string_dict.get(&key).map(|c| c.to_string())
    }

    pub fn info(&self) -> DatabaseInfo {
        DatabaseInfo {
            name: self.name.to_string(),
            attribution_text: self.attribution.clone().map(|c| c.to_string()),
            path: self.path.clone(),
            build_time: self.build_time.to_string(),
            locations: self.locations.len(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "../../frontend/src/bindings/")]
pub struct Location {
    pub latitude: f32,
    pub longitude: f32,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub state: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LocationEncoded {
    pub latitude: f16,
    pub longitude: f16,
    pub city: StringKey,
    pub country_code: StringKey,
    pub state: StringKey,
}
impl Eq for LocationEncoded {}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../../frontend/src/bindings/")]
pub struct DatabaseInfo {
    pub name: String,
    pub attribution_text: Option<String>,
    pub path: Option<PathBuf>,
    pub build_time: String,
    pub locations: usize,
}

fn str_from_byte_record(record: &[u8]) -> Option<String> {
    match record.is_empty() {
        true => None,
        false => Some(String::from_utf8_lossy(record).to_string()),
    }
}

fn location_to_key(lat: f16, lon: f16) -> LocationKey {
    let mut bytes: [u8; 4] = [0; 4];

    let (left, right) = bytes.split_at_mut(2);
    left.copy_from_slice(&lat.to_le_bytes());
    right.copy_from_slice(&lon.to_le_bytes());

    u32::from_le_bytes(bytes)
}
