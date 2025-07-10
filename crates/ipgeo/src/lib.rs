#![doc = include_str!("../README.md")]

use std::{
    io::{Read, Seek, SeekFrom},
    net::{AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr},
    num::{ParseFloatError, ParseIntError},
    str::Utf8Error,
};

use compact_str::CompactString;
use flate2::read::GzDecoder;

mod database;
mod location;

pub use database::{Database, GenericIp, Ipv4Database, Ipv6Database};
pub use location::{Coordinate, Location, LookupInfo};
use serde::{Deserialize, Serialize};

/// Automatically detects the format of the database and parses it.
pub fn from_read(mut source: impl Read + Seek + 'static) -> Result<GenericDatabase> {
    let info = DatabaseType::detect(&mut source)?;

    let read: Box<dyn Read> = if info.is_gzip {
        Box::new(GzDecoder::new(source))
    } else {
        Box::new(source)
    };

    match info.is_ipv6 {
        true => Database::from_read(read, info.is_num).map(GenericDatabase::Ipv6),
        false => Database::from_read(read, info.is_num).map(GenericDatabase::Ipv4),
    }
}

#[derive(PartialEq, Serialize, Deserialize)]
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

struct DatabaseType {
    is_gzip: bool,
    is_num: bool,
    is_ipv6: bool,
}

impl DatabaseType {
    fn detect(mut source: impl Read + Seek) -> Result<Self> {
        const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

        let mut head = [0; 2];
        source.read_exact(&mut head)?;
        source.seek(SeekFrom::Start(0))?;
        let is_gzip = head == GZIP_MAGIC;

        // fill scratch_buff and move to the beginning
        let mut scratch_buff = [0u8; 50];
        if is_gzip {
            let mut decoder = GzDecoder::new(source);
            decoder.read_exact(&mut scratch_buff)?;
            source = decoder.into_inner();
        } else {
            source.read_exact(&mut scratch_buff)?;
        }
        source.seek(SeekFrom::Start(0))?;

        // Read the first record
        let ip_str = CompactString::from_utf8(
            scratch_buff
                .split(|&b| b == b',')
                .next()
                .ok_or(Error::NoRecords)?,
        )?;

        let parsed_ip = ip_str.parse::<IpAddr>();

        let is_num = parsed_ip.is_err();
        let is_u32 = ip_str.parse::<u32>().is_ok();
        let is_u128 = ip_str.parse::<u128>().is_ok();

        if is_num && !is_u32 && !is_u128 {
            return Err(Error::MalformedIp);
        }

        // *most* IPv6 addresses are not valid u32s (?), so we can use that to check for ipv6-num format??
        let is_ipv6 = if is_num {
            !is_u32
        } else {
            parsed_ip.is_ok_and(|ip| ip.is_ipv6())
        };

        Ok(Self {
            is_gzip,
            is_num,
            is_ipv6,
        })
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// All errors that can occur when parsing the database.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("CSV parsing error: {0}")]
    ReadCsv(#[from] csv::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("IP addresses must either be represented as numbers or literal string representations")]
    MalformedIp,
    #[error("CSV file doesn't have enough columns")]
    NotEnoughColumns,
    #[error("Malformed coordinate: {0}")]
    CoordinateParse(#[from] ParseFloatError),
    #[error("Non-utf8 text found: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("Parsing IP integer: {0}")]
    IpNumParse(#[from] ParseIntError),
    #[error("Parsing IP: {0}")]
    IpStrParse(#[from] AddrParseError),
    /// Indexes into the string database are stored in u32s to save space.
    /// I've never found a database where there aren't more than u32::MAX strings, If they're being generated, please contact me and I'll change this to u64.
    #[error("Database has more than u32::MAX unique strings, which is not supported")]
    DatabaseMetadataOverflow,
    #[error("No records found")]
    NoRecords,
}
