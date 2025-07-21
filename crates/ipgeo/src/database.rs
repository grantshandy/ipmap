use std::{
    collections::{HashMap, hash_map::Entry},
    io::{BufReader, Read},
    net::{AddrParseError, Ipv4Addr, Ipv6Addr},
    num::{NonZero, ParseIntError},
    ops::RangeInclusive,
    str::FromStr,
};

use compact_str::CompactString;
use csv::ReaderBuilder;
use heck::ToTitleCase;
use indexmap::IndexSet;
use rangemap::{RangeInclusiveMap, StepLite};
use rustc_hash::FxBuildHasher;
use serde::{Deserialize, Serialize};

use crate::{
    DatabaseTrait, Error, Result,
    location::{Coordinate, CountryCode, Location},
};
use csv_format::*;

/// CSV indexes for city-ipv[4/6][-num].csv format
/// https://github.com/sapics/ip-location-db?tab=readme-ov-file#city-csv-format
pub(crate) mod csv_format {
    pub const NUM_RECORDS: usize = 9;

    pub const IP_RANGE_START_IDX: usize = 0;
    pub const IP_RANGE_END_IDX: usize = 1;
    pub const COUNTRY_CODE_IDX: usize = 2;
    pub const REGION_IDX: usize = 3;
    pub const CITY_IDX: usize = 5;
    pub const LATITUDE_IDX: usize = 7;
    pub const LONGITUDE_IDX: usize = 8;
}

pub type Ipv4Database = Database<Ipv4Addr>;
pub type Ipv6Database = Database<Ipv6Addr>;

/// IpAddr -(map)-> PackedCoordinate -(locations)-> LocationIndices -(strings)-> Location
#[derive(PartialEq, Serialize, Deserialize)]
pub struct Database<Ip: GenericIp> {
    coordinates: RangeInclusiveMap<Ip::Bits, Coordinate>,
    locations: HashMap<Coordinate, LocationIndices, FxBuildHasher>,
    strings: StringDict,
}

impl<Ip: GenericIp> Database<Ip> {
    pub(crate) fn from_read(read: impl Read, is_num: bool) -> Result<Self> {
        let mut db = Self {
            coordinates: RangeInclusiveMap::new(),
            locations: HashMap::default(),
            strings: StringDict::default(),
        };

        let ip_parser = if is_num {
            Ip::bits_from_num_bytes
        } else {
            Ip::bits_from_str_bytes
        };

        for record in ReaderBuilder::new()
            .has_headers(false)
            .from_reader(BufReader::new(read))
            .byte_records()
        {
            let record = record?;

            if record.len() < NUM_RECORDS {
                return Err(Error::NotEnoughColumns);
            }

            let range = RangeInclusive::new(
                ip_parser(&record[IP_RANGE_START_IDX])?,
                ip_parser(&record[IP_RANGE_END_IDX])?,
            );

            let coord = Coordinate {
                lat: CompactString::from_utf8(&record[LATITUDE_IDX])?.parse::<f32>()?,
                lng: CompactString::from_utf8(&record[LONGITUDE_IDX])?.parse::<f32>()?,
            };

            if let Entry::Vacant(entry) = db.locations.entry(coord) {
                entry.insert(LocationIndices {
                    city: db.strings.insert(&record[CITY_IDX]),
                    region: db.strings.insert(&record[REGION_IDX]),
                    country_code: CountryCode::from(&record[COUNTRY_CODE_IDX]),
                });
            }

            db.coordinates.insert(range, coord);
        }

        Ok(db)
    }
}

impl<Ip: GenericIp> DatabaseTrait<Ip> for Database<Ip> {
    fn get_coordinate(&self, ip: Ip) -> Option<Coordinate> {
        self.coordinates.get(&ip.into()).copied()
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.locations.get(&crd).map(|i| i.populate(&self.strings))
    }
}

/// A trait representing either Ipv4Addr or Ipv6Addr for the needs in the database.
pub trait GenericIp: FromStr<Err = AddrParseError> + From<Self::Bits> + Into<Self::Bits> {
    type Bits: FromStr<Err = ParseIntError>
        + StepLite
        + Ord
        + Clone
        + Copy
        + Serialize
        + for<'de> Deserialize<'de>;

    fn bits_from_str_bytes(record: &[u8]) -> Result<Self::Bits> {
        let bits = CompactString::from_utf8(record)?.parse::<Self>()?.into();
        Ok(bits)
    }

    fn bits_from_num_bytes(record: &[u8]) -> Result<Self::Bits> {
        let bits = CompactString::from_utf8(record)?.parse::<Self::Bits>()?;
        Ok(bits)
    }
}

impl GenericIp for Ipv4Addr {
    type Bits = u32;
}

impl GenericIp for Ipv6Addr {
    type Bits = u128;
}

type StringDictKey = NonZero<u32>;

/// A compact database of strings that can store less than u32::MAX items.
#[derive(PartialEq, Eq, Default, Serialize, Deserialize)]
struct StringDict(IndexSet<CompactString, FxBuildHasher>);

impl StringDict {
    pub fn insert(&mut self, item: &[u8]) -> Option<StringDictKey> {
        if item.is_empty() {
            return None;
        }

        let s = CompactString::from_utf8(item).ok()?.to_lowercase();
        let (idx, _) = self.0.insert_full(s);

        NonZero::new((idx + 1) as u32)
    }

    pub fn get(&self, idx: StringDictKey) -> Option<String> {
        self.0
            .get_index((idx.get() - 1) as usize)
            .map(|c| c.to_title_case())
    }
}

/// The city and region are stored as indexes into a string database.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct LocationIndices {
    city: Option<StringDictKey>,
    region: Option<StringDictKey>,
    country_code: CountryCode,
}

impl LocationIndices {
    fn populate(&self, strings: &StringDict) -> Location {
        Location {
            city: self.city.and_then(|i| strings.get(i)),
            region: self.region.and_then(|i| strings.get(i)),
            country_code: self.country_code.to_string(),
        }
    }
}
