use std::collections::HashMap;
use std::fs::File;
use std::hash::{DefaultHasher, Hasher};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

use compact_str::CompactString;
use half::f16;
use rangemap::RangeInclusiveMap;
use time::OffsetDateTime;
use ts_rs::TS;

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Database {
    map: RangeInclusiveMap<u32, LocationEncoded>,
    string_dict: HashMap<u32, CompactString>,
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
            let record = record.expect("deserialize record");

            let (Some(ip_range_start), Some(ip_range_end)) = (
                str_from_byte_record(&record[0]).and_then(|r| r.parse::<u32>().ok()),
                str_from_byte_record(&record[1]).and_then(|r| r.parse::<u32>().ok()),
            ) else {
                return Err(eyre::eyre!("couldn't parse ip ranges"));
            };

            let (Some(latitude), Some(longitude)) = (
                str_from_byte_record(&record[7]).and_then(|s| s.parse::<f16>().ok()),
                str_from_byte_record(&record[8]).and_then(|s| s.parse::<f16>().ok()),
            ) else {
                continue;
            };

            let city = db.hash_and_insert_str(str_from_byte_record(&record[5]));
            let country_code = db.hash_and_insert_str(str_from_byte_record(&record[2]));
            let state = db.hash_and_insert_str(str_from_byte_record(&record[3]));

            db.map.insert(
                ip_range_start..=ip_range_end,
                LocationEncoded {
                    latitude,
                    longitude,
                    city,
                    country_code,
                    state,
                },
            );
        }

        Ok(db)
    }

    fn hash_and_insert_str(&mut self, item: Option<CompactString>) -> u32 {
        match item {
            Some(item) => {
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

                self.string_dict.insert(key, item);

                key
            }
            None => 0,
        }
    }
}

#[allow(dead_code)]
impl Database {
    pub fn get(&self, ip: Ipv4Addr) -> Option<Location> {
        self.map.get(&u32::from(ip)).map(|k| Location {
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
            locations: self.map.len(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, Default, TS)]
#[ts(export, export_to = "../../frontend/src/bindings/")]
pub struct Location {
    pub latitude: f32,
    pub longitude: f32,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub state: Option<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct LocationEncoded {
    pub latitude: f16,
    pub longitude: f16,
    pub city: u32,
    pub country_code: u32,
    pub state: u32,
}
impl Eq for LocationEncoded {}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, TS)]
#[ts(export, export_to = "../../frontend/src/bindings/")]
pub struct DatabaseInfo {
    pub name: String,
    pub attribution_text: Option<String>,
    pub path: Option<PathBuf>,
    pub build_time: String,
    pub locations: usize,
}

fn str_from_byte_record(record: &[u8]) -> Option<CompactString> {
    match record.is_empty() {
        true => None,
        false => CompactString::from_utf8(record).ok(),
    }
}
