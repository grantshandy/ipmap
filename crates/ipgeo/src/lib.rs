#![doc = include_str!("../README.md")]

use std::{
    io::{Read, Seek, SeekFrom},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use compact_str::CompactString;
use flate2::read::GzDecoder;

#[doc(hidden)]
pub use rangemap::StepLite;

mod database;
mod ip_parser;
mod location;

pub use database::{Database, SteppedIp};
pub use location::{Coordinate, Location, LookupInfo};

/// Automatically detects the format of the GeoDatabase and parses it.
pub fn from_read(source: impl Read + Seek + 'static) -> Result<GenericDatabase, Error> {
    let db = DatabaseFile::detect(source)?;

    use ip_parser::{num, str};

    #[rustfmt::skip]
    let parsed = match (db.is_num, db.is_ipv6) {
        (false, false) => Database::from_read(db.read, str::ipv4).map(GenericDatabase::Ipv4),
        (true, false) => Database::from_read(db.read, num::ipv4).map(GenericDatabase::Ipv4),
        (false, true) => Database::from_read(db.read, str::ipv6).map(GenericDatabase::Ipv6),
        (true, true) => Database::from_read(db.read, num::ipv6).map(GenericDatabase::Ipv6),
    };

    parsed
}

#[derive(PartialEq)]
pub enum GenericDatabase {
    Ipv4(Database<Ipv4Addr>),
    Ipv6(Database<Ipv6Addr>),
}

impl GenericDatabase {
    pub fn is_ipv4(&self) -> bool {
        matches!(self, Self::Ipv4(_))
    }

    pub fn is_ipv6(&self) -> bool {
        matches!(self, Self::Ipv6(_))
    }
}

/// CSV indexes for city-ipv[4/6][-num].csv format
/// https://github.com/sapics/ip-location-db?tab=readme-ov-file#city-csv-format
pub(crate) mod csv_format {
    pub const IP_RANGE_START_IDX: usize = 0;
    pub const IP_RANGE_END_IDX: usize = 1;
    pub const COUNTRY_CODE_IDX: usize = 2;
    pub const STATE_IDX: usize = 3;
    pub const CITY_IDX: usize = 5;
    pub const LATITUDE_IDX: usize = 7;
    pub const LONGITUDE_IDX: usize = 8;
}

struct DatabaseFile {
    read: Box<dyn Read>,
    is_num: bool,
    is_ipv6: bool,
}

impl DatabaseFile {
    fn detect(mut source: impl Read + Seek + 'static) -> Result<Self, Error> {
        let is_gzip = is_gzip(&mut source)?;
        let ip_str = first_record(&mut source, is_gzip)?;

        let parsed_ip = ip_str.parse::<IpAddr>();

        let is_num = parsed_ip.is_err();
        let is_u32 = ip_str.parse::<u32>().is_ok();
        let is_u128 = ip_str.parse::<u128>().is_ok();

        // all u32s can be parsed as u128s
        if is_num && !is_u128 {
            return Err(Error::InvalidFormat);
        }

        // *most* IPv6 addresses are not valid u32s (?), so we can use that to check for ipv6-num format??
        let is_ipv6 = if is_num {
            !is_u32
        } else {
            parsed_ip.is_ok_and(|ip| ip.is_ipv6())
        };

        let read: Box<dyn Read> = if is_gzip {
            Box::new(GzDecoder::new(source))
        } else {
            Box::new(source)
        };

        Ok(DatabaseFile {
            read,
            is_num,
            is_ipv6,
        })
    }
}

fn is_gzip(mut source: impl Read + Seek) -> Result<bool, Error> {
    const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

    let mut head = [0; 2];
    source.read_exact(&mut head)?;
    source.seek(SeekFrom::Start(0))?;

    Ok(head == GZIP_MAGIC)
}

fn first_record(mut source: impl Read + Seek, gzip: bool) -> Result<CompactString, Error> {
    // fill scratch_buff and move to the beginning
    let mut scratch_buff = [0u8; 50];
    if gzip {
        let mut decoder = GzDecoder::new(source);
        decoder.read_exact(&mut scratch_buff)?;
        source = decoder.into_inner();
    } else {
        source.read_exact(&mut scratch_buff)?;
    }
    source.seek(SeekFrom::Start(0))?;

    // Read the first record
    scratch_buff
        .split(|&b| b == b',')
        .next()
        .and_then(|line| CompactString::from_utf8(line).ok())
        .ok_or(Error::NoRecords)
}

/// All errors that can occur when parsing the database.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("csv parsing error: {0}")]
    ReadCsv(#[from] csv::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// TODO: add line/column number to this error.
    #[error("error parsing CSV contents")]
    InvalidFormat,
    /// Indexes into the string database are stored in u32s to save space.
    /// I've never found a database where there aren't more than u32::MAX strings, If they're being generated, please contact me and I'll change this to u64.
    #[error("database has more than u32::MAX unique strings, which is not supported")]
    DatabaseMetadataOverflow,
    #[error("no records found")]
    NoRecords,
}
