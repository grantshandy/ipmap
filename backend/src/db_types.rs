use std::io;

use compact_str::CompactString;
use csv::DeserializeError;
use serde::de::Error;

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

pub type GeoDb = rangemap::RangeInclusiveMap<u32, Location>;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Location {
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub city: Option<CompactString>,
    pub country_code: Option<CompactString>,
    pub timezone: Option<CompactString>,
    pub state: Option<CompactString>,
}
impl Eq for Location {}

pub fn read_csv<R: io::Read>(rdr: R) -> Result<(GeoDb, usize), DeserializeError> {
    let mut db = GeoDb::new();

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

            db.insert(
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

    Ok((db, locations))
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
