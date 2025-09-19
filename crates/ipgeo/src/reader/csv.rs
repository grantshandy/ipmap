use std::io::Read;

use compact_str::CompactString;
use treebitmap::IpLookupTable;

use crate::{
    Coordinate, Error, GenericIp,
    locations::{CountryCode, LocationIndices, LocationStore},
};

/// CSV indexes for city-ipv[4/6][-num].csv format
/// https://github.com/sapics/ip-location-db?tab=readme-ov-file#city-csv-format
const NUM_RECORDS: usize = 9;

const IP_RANGE_START_IDX: usize = 0;
const IP_RANGE_END_IDX: usize = 1;
const COUNTRY_CODE_IDX: usize = 2;
const REGION_IDX: usize = 3;
const CITY_IDX: usize = 5;
const LATITUDE_IDX: usize = 7;
const LONGITUDE_IDX: usize = 8;

pub fn read<Ip: GenericIp>(
    read: impl Read,
    is_num: bool,
    ips: &mut IpLookupTable<Ip, Coordinate>,
    locations: &mut LocationStore,
) -> Result<(), crate::Error> {
    let ip_parser = if is_num {
        Ip::from_num_bytes
    } else {
        Ip::from_str_bytes
    };

    for record in csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(read)
        .byte_records()
    {
        let record = record?;

        if record.len() < NUM_RECORDS {
            return Err(Error::NotEnoughColumns);
        }

        let coord = Coordinate {
            lat: CompactString::from_utf8(&record[LATITUDE_IDX])?.parse::<f32>()?,
            lng: CompactString::from_utf8(&record[LONGITUDE_IDX])?.parse::<f32>()?,
        };

        locations.insert(coord, |strings| LocationIndices {
            city: strings.insert_bytes(&record[CITY_IDX]),
            region: strings.insert_bytes(&record[REGION_IDX]),
            country_code: CountryCode::from(&record[COUNTRY_CODE_IDX]),
        });

        for (addr, len) in Ip::range_subnets(
            ip_parser(&record[IP_RANGE_START_IDX])?,
            ip_parser(&record[IP_RANGE_END_IDX])?,
        ) {
            ips.insert(addr, len, coord);
        }
    }

    Ok(())
}
