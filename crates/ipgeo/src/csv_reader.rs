use std::{
    collections::{HashMap, hash_map::Entry},
    io::{BufReader, Read},
    ops::RangeInclusive,
};

use compact_str::CompactString;
use csv::ReaderBuilder;
use csv_format::*;
use rangemap::RangeInclusiveMap;

use crate::{
    Coordinate, Database, Error, GenericIp, Result,
    database::{LocationIndices, StringDict},
    location::CountryCode,
};

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

impl<Ip: GenericIp> Database<Ip> {
    pub(crate) fn from_csv(read: impl Read, is_num: bool) -> Result<Self> {
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
                    city: db.strings.insert_bytes(&record[CITY_IDX]),
                    region: db.strings.insert_bytes(&record[REGION_IDX]),
                    country_code: CountryCode::from(&record[COUNTRY_CODE_IDX]),
                });
            }

            db.coordinates.insert(range, coord);
        }

        Ok(db)
    }
}
