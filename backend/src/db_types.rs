use std::collections::HashMap;
use std::hash::{DefaultHasher, Hasher};
use std::io;
use std::net::Ipv4Addr;
use std::ops::RangeInclusive;

use compact_str::CompactString;
use csv::DeserializeError;
use half::f16;
use serde::de::Error;
use rangemap::RangeInclusiveMap;

#[derive(Clone, PartialEq, Eq, serde::Serialize)]
pub struct Database {
    pub db: GeoDb,
    pub info: Info,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Info {
    pub name: String,
    pub attribution_text: Option<String>,
    pub path: Option<String>,
    pub build_time: String,
    pub locations: usize,
}

#[derive(serde::Deserialize, serde::Serialize, Default, PartialEq, Eq, Clone)]
pub struct GeoDb {
    map: RangeInclusiveMap<u32, LocationEncoded>,
    strings: HashMap<u32, CompactString>,
}

impl GeoDb {
    pub fn add(&mut self, ip_range: RangeInclusive<u32>, item: Location) {
        let (Some(latitude), Some(longitude)) = (item.latitude, item.longitude) else {
            return;
        };

        let city = self.hash_and_insert(item.city);
        let country_code = self.hash_and_insert(item.country_code);
        let timezone = self.hash_and_insert(item.timezone);
        let state = self.hash_and_insert(item.state);

        self.map.insert(
            ip_range,
            LocationEncoded {
                latitude: f16::from_f32(latitude),
                longitude: f16::from_f32(longitude),
                city,
                country_code,
                timezone,
                state,
            },
        );
    }

    fn hash_and_insert(&mut self, item: Option<CompactString>) -> u32 {
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

                if let Some(prev) = self.strings.get(&key) {
                    if prev != &item {
                        println!("strings \"{prev}\" and \"{item}\" collided");
                    }
                }

                self.strings.insert(key, item);

                key
            }
            None => 0,
        }
    }
}

#[allow(dead_code)]
impl GeoDb {
    pub fn get(&self, ip: Ipv4Addr) -> Option<Location> {
        self.map.get(&u32::from(ip)).map(|k| Location {
            latitude: Some(k.latitude.to_f32()),
            longitude: Some(k.longitude.to_f32()),
            city: self.str_from_dict(k.city),
            country_code: self.str_from_dict(k.country_code),
            timezone: self.str_from_dict(k.timezone),
            state: self.str_from_dict(k.state),
        })
    }

    fn str_from_dict(&self, key: u32) -> Option<CompactString> {
        self.strings.get(&key).cloned()
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct Location {
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub city: Option<CompactString>,
    pub country_code: Option<CompactString>,
    pub timezone: Option<CompactString>,
    pub state: Option<CompactString>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct LocationEncoded {
    pub latitude: f16,
    pub longitude: f16,
    pub city: u32,
    pub country_code: u32,
    pub timezone: u32,
    pub state: u32,
}
impl Eq for LocationEncoded {}

pub fn read_csv<R: io::Read>(rdr: R) -> Result<(GeoDb, usize), DeserializeError> {
    let mut db = GeoDb::default();

    let locations = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(rdr)
        .byte_records()
        .map(|record| {
            let record = record.expect("deserialize record");

            let (Ok(ip_range_start), Ok(ip_range_end)) = (
                str_from_byte_record(&record[0]).unwrap().parse::<u32>(),
                str_from_byte_record(&record[1]).unwrap().parse::<u32>(),
            ) else {
                return Err(DeserializeError::custom("couldn't parse ip ranges"));
            };

            db.add(
                ip_range_start..=ip_range_end,
                Location {
                    latitude: str_from_byte_record(&record[7]).and_then(|s| s.parse::<f32>().ok()),
                    longitude: str_from_byte_record(&record[8]).and_then(|s| s.parse::<f32>().ok()),
                    city: str_from_byte_record(&record[5]),
                    country_code: str_from_byte_record(&record[2]),
                    timezone: str_from_byte_record(&record[9]),
                    state: str_from_byte_record(&record[3]),
                },
            );

            Ok(())
        })
        .collect::<Result<Vec<()>, DeserializeError>>()?
        .len();

    return Ok((db, locations));
}

fn str_from_byte_record(record: &[u8]) -> Option<CompactString> {
    match record.is_empty() {
        true => None,
        false => CompactString::from_utf8(record).ok(),
    }
}

pub fn build_time() -> String {
    time::OffsetDateTime::now_utc().to_string()
}
