use std::{
    fmt::{Display, Formatter},
    fs::File,
    hash::{BuildHasherDefault, Hash, Hasher},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::RangeInclusive,
    path::{Path, PathBuf},
    str::FromStr,
};

use compact_str::CompactString;
use half::f16;
use heck::ToTitleCase;
use indexmap::IndexSet;
use rangemap::{RangeInclusiveMap, StepLite};
use rstar::RTree;
use rustc_hash::FxHashMap;
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use ts_rs::TS;

// CSV indexes for ipvx-num.csv format
//
//   https://github.com/sapics/ip-location-db?tab=readme-ov-file#city-csv-format
//
const IP_RANGE_START_IDX: usize = 0;
const IP_RANGE_END_IDX: usize = 1;
const LATITUDE_IDX: usize = 7;
const LONGITUDE_IDX: usize = 8;
const CITY_IDX: usize = 5;
const STATE_IDX: usize = 3;
const COUNTRY_CODE_IDX: usize = 2;

/// A highly memory-efficient map for Ipv4Addr -> Location.
///
/// <Ipv4Addr>
/// --(map)--> <Coordinate>
/// --(locations)--> <LocationDetails>
/// --(strings)--> <Location>
///
#[derive(Debug, Serialize, Deserialize)]
pub struct Database<B: Ord + Clone + StepLite> {
    map: RangeInclusiveMap<B, Coordinate>,
    rev_map: FxHashMap<Coordinate, Vec<IpRange>>,
    locations: FxHashMap<Coordinate, CompactLocationInfo>,
    strings: FxIndexSet<CompactString>,
    coord_tree: RTree<Coordinate>,

    name: CompactString,
    attribution: Option<CompactString>,
    path: Option<PathBuf>,
    build_time: OffsetDateTime,
}

#[allow(dead_code)]
impl<B: Ord + Clone + StepLite + FromStr> Database<B>
where
    IpRange: From<RangeInclusive<B>>,
{
    /// Read a database from a ipv4/6-num.csv
    pub fn from_csv(path: impl AsRef<Path>, attribution: Option<String>) -> eyre::Result<Self> {
        let res = CompactDatabase::from_csv(path, attribution).map(|d| d.into());
        tracing::info!("finished loading db");
        res
    }
}

#[allow(dead_code)]
impl<B: Ord + Clone + StepLite> Database<B> {
    /// Gets the nearest range of ips
    pub fn get_ranges(&self, coord: &Coordinate) -> Vec<IpRange> {
        self.rev_map.get(coord).cloned().unwrap_or_default()
    }

    /// Get the city, state, and country associated with a coordinate.
    pub fn get_location_info(&self, coord: &Coordinate) -> Option<LocationInfo> {
        self.locations
            .get(coord)
            .map(|info| self.decode_location(info))
    }

    /// Create a uncompressed, full LocationInfo
    fn decode_location(&self, info: &CompactLocationInfo) -> LocationInfo {
        LocationInfo {
            city: self
                .strings
                .get_index(info.city as usize)
                .map(|c| c.to_title_case()),
            country_code: info.country_code.to_string(),
            state: self
                .strings
                .get_index(info.state as usize)
                .map(|c| c.to_title_case()),
        }
    }

    /// The nearest [`Coordinate`] in the database
    pub fn nearest_location(&self, coord: &Coordinate) -> Coordinate {
        *self
            .coord_tree
            .nearest_neighbor(coord)
            .expect("empty database")
    }
}

#[allow(dead_code)]
impl Database<Ipv4Bytes> {
    /// Get a coordinate from the ip.
    pub fn get(&self, ip: Ipv4Addr) -> Option<Coordinate> {
        self.map.get(&Ipv4Bytes::from(ip)).copied()
    }

    /// Get the associated range that ip falls into in the database.
    pub fn get_range(&self, ip: Ipv4Addr) -> Option<IpRange> {
        self.map
            .get_key_value(&Ipv4Bytes::from(ip))
            .map(|(r, _)| IpRange::from(r.clone()))
    }

    /// Get the [`DatabaseInfo`] metadata.
    pub fn get_db_info(&self) -> DatabaseInfo {
        DatabaseInfo {
            name: self.name.to_string(),
            kind: IpType::IPv4,
            query: self
                .path
                .clone()
                .map(DatabaseType::Loaded)
                .unwrap_or(DatabaseType::Internal),
            attribution_text: self.attribution.as_ref().map(|c| c.to_string()),
            build_time: self.build_time.to_string(),
            unique_locations: self.locations.len(),
            strings: self.strings.len(),
        }
    }
}

#[allow(dead_code)]
impl Database<Ipv6Bytes> {
    /// Get a coordinate from the ip.
    pub fn get(&self, ip: Ipv6Addr) -> Option<Coordinate> {
        self.map.get(&Ipv6Bytes::from(ip)).copied()
    }

    /// Get the associated range that ip falls into in the database.
    pub fn get_range(&self, ip: Ipv6Addr) -> Option<IpRange> {
        self.map
            .get_key_value(&Ipv6Bytes::from(ip))
            .map(|(r, _)| IpRange::from(r.clone()))
    }

    /// Get the [`DatabaseInfo`] metadata.
    pub fn get_db_info(&self) -> DatabaseInfo {
        DatabaseInfo {
            name: self.name.to_string(),
            kind: IpType::IPv6,
            query: self
                .path
                .clone()
                .map(DatabaseType::Loaded)
                .unwrap_or(DatabaseType::Internal),
            attribution_text: self.attribution.as_ref().map(|c| c.to_string()),
            build_time: self.build_time.to_string(),
            unique_locations: self.locations.len(),
            strings: self.strings.len(),
        }
    }
}

impl<B: Ord + Clone + StepLite> From<CompactDatabase<B>> for Database<B>
where
    IpRange: From<RangeInclusive<B>>,
{
    fn from(val: CompactDatabase<B>) -> Database<B> {
        let map =
            RangeInclusiveMap::from_iter(val.map.iter().map(|(k, v)| (k.clone(), (*v).into())));
        let locations = FxHashMap::from_iter(val.locations.iter().map(|(k, v)| ((*k).into(), *v)));

        let mut coord_tree = RTree::new();
        val.locations
            .keys()
            .for_each(|coord| coord_tree.insert((*coord).into()));

        let mut rev_map: FxHashMap<Coordinate, Vec<IpRange>> = FxHashMap::default();
        val.map
            .iter()
            .map(|(range, coord)| ((*coord).into(), IpRange::from(range.clone())))
            .for_each(|(k, v)| {
                if let Some(c) = rev_map.get_mut(&k) {
                    c.push(v);
                } else {
                    rev_map.insert(k, vec![v]);
                }
            });

        Database {
            map,
            rev_map,
            locations,
            strings: val.strings,
            coord_tree,
            name: val.name,
            attribution: val.attribution,
            path: val.path,
            build_time: val.build_time,
        }
    }
}

/// A variant of [`Database`] which stores only essential information, created to be embedded in the executable at as-small-as-possible sizes.
#[derive(Debug, Serialize, Deserialize)]
pub struct CompactDatabase<B: Ord + Clone + StepLite> {
    map: RangeInclusiveMap<B, TinyCoordinate>,
    locations: FxHashMap<TinyCoordinate, CompactLocationInfo>,
    strings: FxIndexSet<CompactString>,

    name: CompactString,
    attribution: Option<CompactString>,
    path: Option<PathBuf>,
    build_time: OffsetDateTime,
}

impl<B: Ord + Clone + StepLite + FromStr> CompactDatabase<B> {
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
            locations: FxHashMap::default(),
            strings: FxIndexSet::default(),

            name: CompactString::from(name),
            attribution: attribution.map(CompactString::from),
            path,
            build_time: OffsetDateTime::now_utc(),
        };

        let (zero, _) = db.strings.insert_full(CompactString::default());
        assert_eq!(zero, 0);

        for record in csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(file)
            .byte_records()
        {
            let record = record.expect("deserialize byte record");

            let (Some(ip_range_start), Some(ip_range_end)) = (
                str_from_byte_record(&record[IP_RANGE_START_IDX]).and_then(|r| r.parse::<B>().ok()),
                str_from_byte_record(&record[IP_RANGE_END_IDX]).and_then(|r| r.parse::<B>().ok()),
            ) else {
                return Err(eyre::eyre!("couldn't parse ip ranges"));
            };

            let loc_key = match (
                str_from_byte_record(&record[LATITUDE_IDX]).and_then(|s| s.parse::<f16>().ok()),
                str_from_byte_record(&record[LONGITUDE_IDX]).and_then(|s| s.parse::<f16>().ok()),
            ) {
                (Some(latitude), Some(longitude)) => TinyCoordinate {
                    lat: latitude,
                    lng: longitude,
                },
                _ => continue,
            };

            if !db.locations.contains_key(&loc_key) {
                let city = db.hash_and_insert_str(str_from_byte_record(&record[CITY_IDX]));
                let state = db.hash_and_insert_str(str_from_byte_record(&record[STATE_IDX]));
                let country_code = CountryCode::from(
                    str_from_byte_record(&record[COUNTRY_CODE_IDX]).unwrap_or_default(),
                );

                db.locations.insert(
                    loc_key,
                    CompactLocationInfo {
                        city,
                        country_code,
                        state,
                    },
                );
            }

            db.map.insert(ip_range_start..=ip_range_end, loc_key);
        }

        Ok(db)
    }

    fn hash_and_insert_str(&mut self, item: Option<String>) -> u32 {
        match item {
            Some(item) => {
                let idx = self.strings.insert_full(item.to_lowercase().into()).0;

                if idx > u32::MAX as usize {
                    panic!(
                        "Database has more than {} strings, pls contact developer :)",
                        u32::MAX
                    );
                }

                idx as u32
            }
            None => 0, // no value is a zero.
        }
    }
}

fn str_from_byte_record(record: &[u8]) -> Option<String> {
    match record.is_empty() {
        true => None,
        false => Some(String::from_utf8_lossy(record).to_string()),
    }
}

#[allow(dead_code)]
pub type FxIndexSet<T> = IndexSet<T, BuildHasherDefault<FxHasher>>;

#[allow(dead_code)]
pub type Ipv4Bytes = u32;

#[allow(dead_code)]
pub type Ipv6Bytes = u128;

/// A compact representation of [`LocationInfo`] which uses indexes into [`Database::strings`] instead of the strings themselves.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompactLocationInfo {
    pub city: u32,
    pub state: u32,
    pub country_code: CountryCode,
}

/// A generalized lat/lon coordinate type which seeps all the way up the stack to leaflet's `LatLngExpression`.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct GenCoordinate<C> {
    pub lat: C,
    pub lng: C,
}

pub type TinyCoordinate = GenCoordinate<f16>;
pub type Coordinate = GenCoordinate<f32>;

// using floating point numbers in maps
// throughout the application is *concerning*,
// but we should be never doing any arithmetic
// on them and our database should never produce
// f16/32::NAN, so we should be safe to `Hash`
// and `Eq` them with their raw bytes.

impl<C: PartialEq> Eq for GenCoordinate<C> {}

impl Hash for Coordinate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.lat.to_ne_bytes().hash(state);
        self.lng.to_ne_bytes().hash(state);
    }
}

impl Hash for TinyCoordinate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.lat.to_ne_bytes().hash(state);
        self.lng.to_ne_bytes().hash(state);
    }
}

impl From<TinyCoordinate> for Coordinate {
    fn from(val: TinyCoordinate) -> Coordinate {
        Coordinate {
            lat: val.lat.to_f32(),
            lng: val.lng.to_f32(),
        }
    }
}

impl From<Coordinate> for TinyCoordinate {
    fn from(val: Coordinate) -> TinyCoordinate {
        TinyCoordinate {
            lat: f16::from_f32(val.lat),
            lng: f16::from_f32(val.lng),
        }
    }
}

/// There is a 'num-traits' feature for
/// `half` (our f16 impl provider), but
/// it doesn't implement all the traits
/// that we would need to impl rstar::Point
///
/// This leads to a major PITA, converting
/// to f32s for the main in-memory database
/// representation, which takes up significantly
/// more data.
///
impl rstar::Point for Coordinate {
    type Scalar = f32;
    const DIMENSIONS: usize = 2;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        Self {
            lat: generator(0),
            lng: generator(1),
        }
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.lat,
            1 => self.lng,
            _ => unreachable!(),
        }
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.lat,
            1 => &mut self.lng,
            _ => unreachable!(),
        }
    }
}

/// An ISO 3166 2-digit ASCII country code
/// which takes advantage of their compact representation :)
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct CountryCode([u8; 2]);

impl<A: AsRef<[u8]>> From<A> for CountryCode {
    fn from(value: A) -> Self {
        match value.as_ref() {
            [a, b, ..] => Self([*a, *b]),
            _ => Self([0, 0]),
        }
    }
}

impl Display for CountryCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            [0, 0] => "??".fmt(f),
            _ => unsafe {
                char::from_u32_unchecked(self.0[0] as u32).fmt(f)?;
                char::from_u32_unchecked(self.0[1] as u32).fmt(f)
            },
        }
    }
}

/// A location and its associated IpRanges
#[derive(Clone, Debug, PartialEq, Serialize, Default, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct LocationBlock {
    pub location: LocationInfo,
    pub blocks: Vec<IpRange>,
}

/// A simplified RangeInclusive<Ipv4Addr/u32> for our TS api.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct IpRange {
    pub lower: IpAddr,
    pub upper: IpAddr,
}

impl From<RangeInclusive<Ipv4Bytes>> for IpRange {
    fn from(value: RangeInclusive<Ipv4Bytes>) -> Self {
        Self {
            lower: IpAddr::V4(Ipv4Addr::from(*value.start())),
            upper: IpAddr::V4(Ipv4Addr::from(*value.end())),
        }
    }
}

impl From<RangeInclusive<Ipv6Bytes>> for IpRange {
    fn from(value: RangeInclusive<Ipv6Bytes>) -> Self {
        Self {
            lower: IpAddr::V6(Ipv6Addr::from(*value.start())),
            upper: IpAddr::V6(Ipv6Addr::from(*value.end())),
        }
    }
}

/// Associated metadata for a certain coordinate in the database.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct LocationInfo {
    pub city: Option<String>,
    pub country_code: String,
    pub state: Option<String>,
}

/// The associated metadata for a given Database
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct DatabaseInfo {
    pub name: String,
    pub kind: IpType,
    pub query: DatabaseType,
    pub attribution_text: Option<String>,
    pub build_time: String,
    pub unique_locations: usize,
    pub strings: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub enum IpType {
    IPv4,
    IPv6,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub enum DatabaseType {
    Loaded(PathBuf),
    Internal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct DatabaseQuery {
    pub ipv4: Option<DatabaseType>,
    pub ipv6: Option<DatabaseType>,
}
