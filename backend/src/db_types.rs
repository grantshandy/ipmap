use std::io;

use compact_str::CompactString;
use csv::DeserializeError;

#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct DatabaseInfo {
    pub filename: &'static str,
    pub built: &'static str,
    pub attribution: &'static str
}

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

pub type GeoDb = rangemap::RangeInclusiveMap<u32, Location>;

pub fn read_csv<R: io::Read>(rdr: R) -> Result<GeoDb, DeserializeError> {
    let mut db = GeoDb::new();

    csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(rdr)
        .byte_records()
        .for_each(|record| {
            let record = record.expect("deserialize record");

            let ip_range_start = str_from_byte_record(&record[0])
                .unwrap()
                .parse::<u32>()
                .unwrap();
            let ip_range_end = str_from_byte_record(&record[1])
                .unwrap()
                .parse::<u32>()
                .unwrap();
            let country_code = str_from_byte_record(&record[2]);
            let state = str_from_byte_record(&record[3]);
            let city = str_from_byte_record(&record[5]);
            let latitude = str_from_byte_record(&record[7]).map(|s| s.parse::<f32>().unwrap());
            let longitude = str_from_byte_record(&record[8]).map(|s| s.parse::<f32>().unwrap());
            let timezone = str_from_byte_record(&record[9]);

            db.insert(
                ip_range_start..=ip_range_end,
                Location {
                    latitude,
                    longitude,
                    city,
                    country_code,
                    timezone,
                    state,
                },
            );
        });

    Ok(db)
}

fn str_from_byte_record(record: &[u8]) -> Option<CompactString> {
    match record.is_empty() {
        true => None,
        false => CompactString::from_utf8(record).ok(),
    }
}
