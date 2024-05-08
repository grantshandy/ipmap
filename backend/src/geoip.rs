use std::{
    fs::File,
    net::Ipv4Addr,
    ops::Deref,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use csv::ReaderBuilder;
use rangemap::RangeInclusiveMap;
use tauri::State;

pub type GeoDb = RangeInclusiveMap<u32, Location>;
pub type RuntimeDb = Arc<RwLock<Option<GeoDb>>>;

#[tauri::command]
pub async fn lookup_ip(
    state: State<'_, RuntimeDb>,
    ip: String,
) -> Result<Option<Location>, String> {
    tracing::info!("looking up {ip}");

    match state.read().expect("read state").deref() {
        Some(db) => {
            let ip = ip.parse::<Ipv4Addr>().map_err(|e| e.to_string())?;

            Ok(db.get(&u32::from(ip)).cloned())
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn set_database(
    state: State<'_, RuntimeDb>,
    path: Option<PathBuf>,
) -> Result<Option<PathBuf>, String> {
    let Some(path) = path else {
        *state.write().expect("write db") = None;
        tracing::info!("cleared database state");
        return Ok(None);
    };

    tracing::info!("parsing database at {path:?}");

    let mut db = GeoDb::new();

    ReaderBuilder::new()
        .has_headers(false)
        .from_reader(File::open(&path).map_err(|e| e.to_string())?)
        .deserialize::<CityRecordIpv4Num>()
        .map(|record| {
            record.map(|record| {
                db.insert(record.ip_range_start..=record.ip_range_end, record.into());
            })
        })
        .collect::<Result<(), csv::Error>>()
        .map_err(|e| e.to_string())?;

    tracing::info!("finished parsing database");

    *state.write().expect("write db") = Some(db);

    Ok(Some(path))
}

#[derive(Clone, serde::Deserialize)]
struct CityRecordIpv4Num {
    ip_range_start: u32,
    ip_range_end: u32,
    country_code: Option<String>,
    state1: Option<String>,
    _state2: Option<String>,
    city: Option<String>,
    _postcode: Option<String>,
    latitude: Option<f32>,
    longitude: Option<f32>,
    timezone: Option<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Location {
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub timezone: Option<String>,
    pub state: Option<String>,
}
impl Eq for Location {}

impl From<CityRecordIpv4Num> for Location {
    fn from(other: CityRecordIpv4Num) -> Self {
        Self {
            latitude: other.latitude,
            longitude: other.longitude,
            city: other.city,
            country_code: other.country_code,
            timezone: other.timezone,
            state: other.state1,
        }
    }
}
