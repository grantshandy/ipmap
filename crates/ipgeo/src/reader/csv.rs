use std::io::Read;

use compact_str::CompactString;
use csv::ByteRecord;
use treebitmap::IpLookupTable;

use crate::{
    Coordinate, Error, GenericIp,
    locations::{CountryCode, LocationIndices, LocationStore},
};

/// CSV indexes for city-ipv[4/6][-num].csv format
/// https://github.com/sapics/ip-location-db?tab=readme-ov-file#city-csv-format
pub const NUM_RECORDS: usize = 9;

pub const IP_RANGE_START_IDX: usize = 0;
pub const IP_RANGE_END_IDX: usize = 1;
pub const COUNTRY_CODE_IDX: usize = 2;
pub const REGION_IDX: usize = 3;
pub const CITY_IDX: usize = 5;
pub const LATITUDE_IDX: usize = 7;
pub const LONGITUDE_IDX: usize = 8;

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
        read_record(&record?, ip_parser, ips, locations)?;
    }

    Ok(())
}

pub fn coord_from_record(record: &ByteRecord) -> Result<Coordinate, crate::Error> {
    Ok(Coordinate {
        lat: CompactString::from_utf8(&record[LATITUDE_IDX])?.parse::<f32>()?,
        lng: CompactString::from_utf8(&record[LONGITUDE_IDX])?.parse::<f32>()?,
    })
}

pub fn read_record<Ip: GenericIp>(
    record: &ByteRecord,
    ip_parser: fn(&[u8]) -> Result<Ip, crate::Error>,
    ips: &mut IpLookupTable<Ip, Coordinate>,
    locations: &mut LocationStore,
) -> Result<(), crate::Error> {
    if record.len() < NUM_RECORDS {
        return Err(Error::NotEnoughColumns);
    }

    let coord = coord_from_record(record)?;

    locations.insert(coord, &|strings| {
        Ok(LocationIndices {
            city: strings.insert_bytes(&record[CITY_IDX]),
            region: strings.insert_bytes(&record[REGION_IDX]),
            country_code: CountryCode::from(&record[COUNTRY_CODE_IDX]),
        })
    })?;

    for (addr, len) in Ip::range_subnets(
        ip_parser(&record[IP_RANGE_START_IDX])?,
        ip_parser(&record[IP_RANGE_END_IDX])?,
    ) {
        ips.insert(addr, len, coord);
    }

    Ok(())
}
