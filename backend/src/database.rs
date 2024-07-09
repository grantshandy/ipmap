use std::{
    fmt::{Display, Formatter},
    fs::File,
    hash::{BuildHasherDefault, Hash},
    net::Ipv4Addr,
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

use compact_str::CompactString;
use heck::ToTitleCase;
use indexmap::IndexSet;
use rangemap::RangeInclusiveMap;
use rstar::RTree;
use rustc_hash::{FxHashMap, FxHasher};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use ts_rs::TS;

type FxIndexSet<T> = IndexSet<T, BuildHasherDefault<FxHasher>>;
type Ipv4Bytes = u32;

/// A highly memory-efficient map for Ipv4Addr -> Location.
///
/// <Ipv4Addr>
/// --(map)--> <Coordinate>
/// --(locations)--> <LocationDetails>
/// --(strings)--> <Location>
///
#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    map: RangeInclusiveMap<Ipv4Bytes, Coordinate>,
    rev_map: FxHashMap<Coordinate, Vec<IpRange>>,
    locations: FxHashMap<Coordinate, LocationDetails>,
    strings: FxIndexSet<CompactString>,
    coord_tree: RTree<Coordinate>,

    name: CompactString,
    attribution: Option<CompactString>,
    path: Option<PathBuf>,
    build_time: OffsetDateTime,
}

#[allow(dead_code)]
impl Database {
    pub fn from_csv(path: impl AsRef<Path>, attribution: Option<String>) -> eyre::Result<Self> {
        CompactDatabase::from_csv(path, attribution).map(|d| d.into())
    }

    pub fn get(&self, ip: Ipv4Addr) -> Option<Location> {
        self.map
            .get(&u32::from(ip))
            .and_then(|k| self.locations.get(k).map(|l| (k, l)))
            .map(|(k, l)| self.decode_location(*k, l))
    }

    pub fn get_range(&self, ip: Ipv4Addr) -> Option<RangeInclusive<Ipv4Addr>> {
        self.map.get_key_value(&u32::from(ip)).map(|(range, _)| {
            RangeInclusive::new(Ipv4Addr::from(*range.start()), Ipv4Addr::from(*range.end()))
        })
    }

    fn decode_location(&self, k: Coordinate, l: &LocationDetails) -> Location {
        Location {
            latitude: k.0,
            longitude: k.1,
            city: self
                .strings
                .get_index(l.city as usize)
                .map(|c| c.to_title_case()),
            country_code: l.country_code.to_string(),
            state: self
                .strings
                .get_index(l.state as usize)
                .map(|c| c.to_title_case()),
        }
    }

    pub fn info(&self) -> DatabaseInfo {
        DatabaseInfo {
            name: self.name.to_string(),
            attribution_text: self.attribution.clone().map(|c| c.to_string()),
            path: self.path.clone(),
            build_time: self.build_time.to_string(),
            unique_locations: self.locations.len(),
            strings: self.strings.len(),
        }
    }

    pub fn nearest_location(&self, lat: f32, lon: f32) -> LocationBlock {
        let nearest = self
            .coord_tree
            .nearest_neighbor(&Coordinate(lat, lon))
            .expect("empty database");

        let location = self
            .locations
            .get(nearest)
            .map(|l| (nearest, l))
            .map(|(k, l)| self.decode_location(*k, l))
            .expect("no location for nearest coord");

        let blocks = self.rev_map.get(nearest).expect("no blocks").clone();

        LocationBlock { location, blocks }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "../../frontend/src/bindings/")]
pub struct Location {
    pub latitude: f32,
    pub longitude: f32,
    pub city: Option<String>,
    pub country_code: String,
    pub state: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../../frontend/src/bindings/")]
pub struct DatabaseInfo {
    pub name: String,
    pub attribution_text: Option<String>,
    pub path: Option<PathBuf>,
    pub build_time: String,
    pub unique_locations: usize,
    pub strings: usize,
}

/// The database stored in the executable,
/// as information dense as possible.
///
/// TODO: use f16 for coords.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct CompactDatabase {
    map: RangeInclusiveMap<Ipv4Bytes, Coordinate>,
    locations: FxHashMap<Coordinate, LocationDetails>,
    strings: FxIndexSet<CompactString>,

    name: CompactString,
    attribution: Option<CompactString>,
    path: Option<PathBuf>,
    build_time: OffsetDateTime,
}

impl CompactDatabase {
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
                    country_code: str_from_byte_record(&record[2]).unwrap_or_default(),
                    state: str_from_byte_record(&record[3]),
                },
            );
        }

        Ok(db)
    }

    fn insert(&mut self, ip_range: RangeInclusive<Ipv4Bytes>, location: Location) {
        let loc_key = Coordinate(location.latitude, location.longitude);

        let city = self.hash_and_insert_str(location.city);
        let state = self.hash_and_insert_str(location.state);

        self.locations.insert(
            loc_key,
            LocationDetails {
                city,
                country_code: location.country_code.into(),
                state,
            },
        );

        self.map.insert(ip_range, loc_key);
    }

    fn hash_and_insert_str(&mut self, item: Option<String>) -> u32 {
        item.map(|item| self.strings.insert_full(item.to_lowercase().into()).0 as u32)
            .unwrap_or(0) // no value is a zero.
    }
}

impl Into<Database> for CompactDatabase {
    fn into(self) -> Database {
        let mut coord_tree = RTree::new();

        self.locations
            .keys()
            .for_each(|coord| coord_tree.insert(*coord));

        let mut rev_map: FxHashMap<Coordinate, Vec<IpRange>> = FxHashMap::default();

        self.map
            .iter()
            .map(|(range, coord)| {
                (
                    *coord,
                    IpRange {
                        lower: Ipv4Addr::from(*range.start()),
                        upper: Ipv4Addr::from(*range.end()),
                    },
                )
            })
            .for_each(|(k, v)| {
                if let Some(c) = rev_map.get_mut(&k) {
                    c.push(v);
                } else {
                    rev_map.insert(k, vec![v]);
                }
            });

        Database {
            map: self.map,
            rev_map,
            locations: self.locations,
            strings: self.strings,
            coord_tree,
            name: self.name,
            attribution: self.attribution,
            path: self.path,
            build_time: self.build_time,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct LocationDetails {
    pub city: u32,
    pub state: u32,
    pub country_code: CountryCode,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Coordinate(f32, f32);

impl Eq for Coordinate {}

// eff you :)
impl Hash for Coordinate {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ne_bytes().hash(state);
        self.1.to_ne_bytes().hash(state);
    }
}

impl rstar::Point for Coordinate {
    type Scalar = f32;
    const DIMENSIONS: usize = 2;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        Self(generator(0), generator(1))
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.0,
            1 => self.1,
            _ => unreachable!(),
        }
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            _ => unreachable!(),
        }
    }
}

/// An ISO 3166 2-digit ASCII Country Code
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[repr(transparent)]
struct CountryCode([u8; 2]);

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

/// A location and its associated Ip Ranges
#[derive(Clone, Debug, PartialEq, Serialize, Default, TS)]
#[ts(export, export_to = "../../frontend/src/bindings/")]
pub struct LocationBlock {
    pub location: Location,
    pub blocks: Vec<IpRange>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/bindings/")]
pub struct IpRange {
    lower: Ipv4Addr,
    upper: Ipv4Addr,
}

impl From<RangeInclusive<Ipv4Addr>> for IpRange {
    fn from(value: RangeInclusive<Ipv4Addr>) -> Self {
        Self {
            lower: *value.start(),
            upper: *value.end(),
        }
    }
}

fn str_from_byte_record(record: &[u8]) -> Option<String> {
    match record.is_empty() {
        true => None,
        false => Some(String::from_utf8_lossy(record).to_string()),
    }
}

#[cfg(test)]
mod test {
    use super::CountryCode;

    #[test]
    fn countrycode() {
        assert_eq!(std::mem::size_of::<CountryCode>(), 2);
        assert_eq!(CountryCode::from("USa").to_string(), "US");
    }
}
