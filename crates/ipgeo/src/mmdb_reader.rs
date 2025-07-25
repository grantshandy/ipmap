use std::{
    collections::{HashMap, hash_map::Entry},
    ops::RangeInclusive,
};

use compact_str::CompactString;
use maxminddb::Reader;
use rangemap::RangeInclusiveMap;
use serde::Deserialize;

use crate::{
    Coordinate, Database, Error, GenericIp, Result,
    database::{LocationIndices, StringDict},
    location::CountryCode,
};

impl<Ip: GenericIp> Database<Ip> {
    pub fn from_mmdb<S: AsRef<[u8]>>(reader: Reader<S>) -> Result<Self> {
        let mut db = Self {
            coordinates: RangeInclusiveMap::new(),
            locations: HashMap::default(),
            strings: StringDict::default(),
        };

        for res in reader
            .within::<CityFormat>(Ip::full_network())
            .map_err(Error::MaxMindDb)?
        {
            let location = res.map_err(Error::MaxMindDb)?;

            let range = RangeInclusive::new(
                Ip::bits_from_generic(location.ip_net.ip()).ok_or(Error::MalformedMaxMindDb)?,
                Ip::bits_from_generic(location.ip_net.broadcast())
                    .ok_or(Error::MalformedMaxMindDb)?,
            );

            let coord = Coordinate {
                lat: location.info.latitude,
                lng: location.info.longitude,
            };

            if let Entry::Vacant(entry) = db.locations.entry(coord) {
                entry.insert(LocationIndices {
                    city: db.strings.insert_str(location.info.city),
                    region: db.strings.insert_str(location.info.state1),
                    country_code: CountryCode::from(location.info.country_code),
                });
            }

            db.coordinates.insert(range, coord);
        }

        Ok(db)
    }
}

/// https://github.com/sapics/ip-location-db/tree/main/dbip-city-mmdb#mmdb-format
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct CityFormat {
    country_code: CompactString,
    city: CompactString,
    state1: CompactString,
    state2: CompactString,
    postcode: CompactString,
    latitude: f32,
    longitude: f32,
    timezone: CompactString,
}
