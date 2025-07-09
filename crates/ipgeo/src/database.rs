use std::{
    collections::HashMap,
    io::Read,
    net::{Ipv4Addr, Ipv6Addr},
    num::NonZero,
    ops::RangeInclusive,
};

use compact_str::CompactString;
use csv::{ByteRecord, ReaderBuilder};
use heck::ToTitleCase;
use indexmap::IndexSet;
use rangemap::{RangeInclusiveMap, StepLite};

use crate::{
    Error, csv_format,
    ip_parser::IpParser,
    location::{Coordinate, CountryCode, Location, LookupInfo},
};

/// IpAddr -(map)-> PackedCoordinate -(locations)-> LocationIndices -(strings)-> Location
#[derive(PartialEq)]
pub struct Database<B> {
    map: RangeInclusiveMap<SteppedIp<B>, Coordinate>,
    locations: HashMap<Coordinate, LocationIndices>,
    strings: StringDict,
}

impl<B> Database<B>
where
    B: Ord + Clone,
    SteppedIp<B>: StepLite,
{
    pub(crate) fn from_read(file: impl Read, parser: IpParser<B>) -> Result<Self, Error> {
        let mut db = Self {
            map: RangeInclusiveMap::new(),
            locations: HashMap::new(),
            strings: StringDict::default(),
        };

        for record in ReaderBuilder::new()
            .has_headers(false)
            .from_reader(file)
            .byte_records()
        {
            let record = record.map_err(Error::ReadCsv)?;

            let range = RangeInclusive::new(
                parser(&record[csv_format::IP_RANGE_START_IDX]).ok_or(Error::InvalidFormat)?,
                parser(&record[csv_format::IP_RANGE_END_IDX]).ok_or(Error::InvalidFormat)?,
            );

            let coord = Coordinate::try_from(&record)?;

            db.map.insert(range, coord);

            #[allow(clippy::map_entry)] // need mutable access to db in that time
            if !db.locations.contains_key(&coord) {
                db.locations.insert(
                    coord,
                    LocationIndices::from_byte_record(&record, &mut db.strings),
                );
            }
        }

        Ok(db)
    }

    pub fn get_coordinate(&self, ip: B) -> Option<Coordinate> {
        self.map.get(&SteppedIp::<B>(ip)).copied()
    }

    pub fn get(&self, ip: B) -> Option<LookupInfo> {
        let crd = self.get_coordinate(ip)?;
        let loc = self
            .locations
            .get(&crd)
            .map(|i| i.populate(&self.strings))?;

        Some(LookupInfo { crd, loc })
    }
}

type StringDictKey = NonZero<u32>;

/// A compact database of strings that can store less than u32::MAX items.
#[derive(PartialEq, Eq, Default)]
struct StringDict(IndexSet<CompactString>);

impl StringDict {
    pub fn insert(&mut self, item: &[u8]) -> Option<StringDictKey> {
        if item.is_empty() {
            return None;
        }

        let s = CompactString::from_utf8_lossy(item).to_lowercase();

        let (idx, _) = self.0.insert_full(s);

        NonZero::new((idx + 1) as u32)
    }

    pub fn get(&self, idx: StringDictKey) -> Option<String> {
        self.0
            .get_index((idx.get() - 1) as usize)
            .map(|c| c.to_title_case())
    }
}

/// A wrapper around Ipv4Addr and Ipv6Addr to allow for a StepLite implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
#[repr(transparent)]
#[doc(hidden)]
pub struct SteppedIp<B>(pub B);

impl rangemap::StepLite for SteppedIp<Ipv4Addr> {
    fn add_one(&self) -> Self {
        Self(self.0.to_bits().add_one().into())
    }

    fn sub_one(&self) -> Self {
        Self(self.0.to_bits().sub_one().into())
    }
}

impl rangemap::StepLite for SteppedIp<Ipv6Addr> {
    fn add_one(&self) -> Self {
        Self(self.0.to_bits().add_one().into())
    }

    fn sub_one(&self) -> Self {
        Self(self.0.to_bits().sub_one().into())
    }
}

/// The city and region are stored as indexes into a string database.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct LocationIndices {
    city: Option<StringDictKey>,
    region: Option<StringDictKey>,
    country_code: CountryCode,
}

impl LocationIndices {
    pub fn from_byte_record(record: &ByteRecord, strings: &mut StringDict) -> Self {
        let city = strings.insert(&record[csv_format::CITY_IDX]);
        let region = strings.insert(&record[csv_format::STATE_IDX]);
        let country_code = CountryCode::from(&record[csv_format::COUNTRY_CODE_IDX]);

        Self {
            city,
            region,
            country_code,
        }
    }

    pub fn populate(&self, strings: &StringDict) -> Location {
        Location {
            city: self.city.and_then(|i| strings.get(i)),
            region: self.region.and_then(|i| strings.get(i)),
            country_code: self.country_code.to_string(),
        }
    }
}
