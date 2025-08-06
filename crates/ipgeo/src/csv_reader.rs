use std::{
    collections::{HashMap, hash_map::Entry},
    io::{BufReader, Read},
    ops::RangeInclusive,
};

use compact_str::CompactString;
use csv::ReaderBuilder;
use csv_format::*;
use treebitmap::IpLookupTable;

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
            coordinates: IpLookupTable::new(),
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

            let (lower, masklen) = Ip::bit_range_to_network(range);
            db.coordinates.insert(lower, masklen, coord);
        }

        Ok(db)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Coordinate, DatabaseTrait, Ipv4Database, Ipv6Database, Location, LookupInfo};
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn parse() {
        let info = LookupInfo {
            crd: Coordinate {
                lat: 23.1317,
                lng: 113.266,
            },
            loc: Location {
                city: Some("Guangzhou".to_string()),
                region: Some("Guangdong".to_string()),
                country_code: "CN".to_string(),
            },
        };

        let format_csv_line = |lower: String, higher: String| {
            format!(
                "{lower},{higher},{},{},,{},,{},{},",
                info.loc.country_code.clone(),
                info.loc.region.clone().unwrap_or_default(),
                info.loc.city.clone().unwrap_or_default(),
                info.crd.lat,
                info.crd.lng
            )
        };

        let ipv4_lower: Ipv4Addr = "1.0.8.0".parse().unwrap();
        let ipv4_higher: Ipv4Addr = "1.0.15.255".parse().unwrap();
        let ipv4_valid: Ipv4Addr = "1.0.9.80".parse().unwrap();
        let ipv4_invalid: Ipv4Addr = "19.0.9.80".parse().unwrap();

        let ipv4_db = Ipv4Database::from_csv(
            format_csv_line(ipv4_lower.to_string(), ipv4_higher.to_string()).as_bytes(),
            false,
        )
        .unwrap();

        assert_eq!(Some(info.clone()), ipv4_db.get(ipv4_valid));
        assert_eq!(None, ipv4_db.get(ipv4_invalid));

        let ipv4_num_db = Ipv4Database::from_csv(
            format_csv_line(
                ipv4_lower.to_bits().to_string(),
                ipv4_higher.to_bits().to_string(),
            )
            .as_bytes(),
            true,
        )
        .unwrap();

        assert_eq!(Some(info.clone()), ipv4_num_db.get(ipv4_valid));
        assert_eq!(None, ipv4_num_db.get(ipv4_invalid));

        let ipv6_lower: Ipv6Addr = "2001:2::".parse().unwrap();
        let ipv6_higher: Ipv6Addr = "2001:2::ffff:ffff:ffff:ffff:ffff".parse().unwrap();
        let ipv6_valid: Ipv6Addr = "2001:2::9".parse().unwrap();
        let ipv6_invalid: Ipv6Addr = "3001:2::9".parse().unwrap();

        let ipv6_db = Ipv6Database::from_csv(
            format_csv_line(ipv6_lower.to_string(), ipv6_higher.to_string()).as_bytes(),
            false,
        )
        .unwrap();

        assert_eq!(Some(info.clone()), ipv6_db.get(ipv6_valid));
        assert_eq!(None, ipv6_db.get(ipv6_invalid));

        let ipv6_num_db = Ipv6Database::from_csv(
            format_csv_line(
                ipv6_lower.to_bits().to_string(),
                ipv6_higher.to_bits().to_string(),
            )
            .as_bytes(),
            true,
        )
        .unwrap();

        assert_eq!(Some(info.clone()), ipv6_num_db.get(ipv6_valid));
        assert_eq!(None, ipv6_num_db.get(ipv6_invalid));
    }
}
