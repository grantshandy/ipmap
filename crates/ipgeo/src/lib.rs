#![doc = include_str!("../README.md")]

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    net::{AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr},
    num::{ParseFloatError, ParseIntError},
    path::Path,
    str::Utf8Error,
};

use compact_str::CompactString;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};

mod csv_reader;
mod database;
mod location;
mod mmdb_reader;

#[cfg(test)]
mod tests;

pub use database::{Database, GenericIp, Ipv4Database, Ipv6Database};
pub use location::{Coordinate, Location, LookupInfo};

/// Automatically detects the format of the database and parses it.
pub fn detect(path: &Path) -> Result<GenericDatabase> {
    match DatabaseType::detect(path)? {
        DatabaseType::Csv {
            reader,
            is_num,
            is_ipv6,
        } => match is_ipv6 {
            true => Database::from_csv(reader, is_num).map(GenericDatabase::Ipv6),
            false => Database::from_csv(reader, is_num).map(GenericDatabase::Ipv4),
        },
        DatabaseType::Maxminddb { reader } => match reader.metadata.ip_version {
            4 => Database::from_mmdb(reader).map(GenericDatabase::Ipv4),
            6 => Database::from_mmdb(reader).map(GenericDatabase::Ipv6),
            _ => Err(Error::MalformedMaxMindDb),
        },
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

impl DatabaseTrait<IpAddr> for GenericDatabase {
    fn get_coordinate(&self, ip: IpAddr) -> Option<Coordinate> {
        match (ip, self) {
            (IpAddr::V4(ip), GenericDatabase::Ipv4(db)) => db.get_coordinate(ip),
            (IpAddr::V6(ip), GenericDatabase::Ipv6(db)) => db.get_coordinate(ip),
            _ => None,
        }
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        match self {
            GenericDatabase::Ipv4(db) => db.get_location(crd),
            GenericDatabase::Ipv6(db) => db.get_location(crd),
        }
    }
}

enum DatabaseType {
    Csv {
        reader: Box<dyn Read>,
        is_num: bool,
        is_ipv6: bool,
    },
    Maxminddb {
        reader: maxminddb::Reader<Vec<u8>>,
    },
}

impl DatabaseType {
    fn detect(path: &Path) -> Result<Self> {
        match path.extension().and_then(|s| s.to_str()) {
            Some("mmdb") => Self::parse_mmdb(path),
            _ => Self::parse_csv(File::open(path)?),
        }
    }

    fn parse_mmdb(path: &Path) -> Result<Self> {
        match maxminddb::Reader::open_readfile(path) {
            Ok(reader) => Ok(Self::Maxminddb { reader }),
            Err(e) => Err(Error::MaxMindDb(e)),
        }
    }

    fn parse_csv(mut f: File) -> Result<Self> {
        const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

        let mut head = [0; 2];
        f.read_exact(&mut head)?;
        f.seek(SeekFrom::Start(0))?;
        let is_gzip = head == GZIP_MAGIC;

        // fill scratch_buff and move to the beginning
        let mut scratch_buff = [0u8; 50];
        if is_gzip {
            let mut decoder = GzDecoder::new(f);
            decoder.read_exact(&mut scratch_buff)?;
            f = decoder.into_inner();
        } else {
            f.read_exact(&mut scratch_buff)?;
        }
        f.seek(SeekFrom::Start(0))?;

        let Some(Ok(ip_str)) = scratch_buff
            .split(|&b| b == b',')
            .next()
            .map(CompactString::from_utf8)
        else {
            return Err(Error::NoRecords);
        };

        let parsed_ip = ip_str.parse::<IpAddr>();

        let is_num = parsed_ip.is_err();
        let is_u32 = ip_str.parse::<u32>().is_ok();
        let is_u128 = ip_str.parse::<u128>().is_ok();

        if is_num && !is_u32 && !is_u128 {
            return Err(Error::InvalidFormat);
        }

        // *most* IPv6 addresses are not valid u32s (?), so we can use that to check for ipv6-num format??
        let is_ipv6 = if is_num {
            !is_u32
        } else {
            parsed_ip.is_ok_and(|ip| ip.is_ipv6())
        };

        let reader: Box<dyn Read> = if is_gzip {
            Box::new(GzDecoder::new(f))
        } else {
            Box::new(f)
        };

        Ok(Self::Csv {
            reader,
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
    #[error("MaxmindDB error: {0}")]
    MaxMindDb(maxminddb::MaxMindDbError),
    #[error("Invalid Maxminddb database")]
    MalformedMaxMindDb,
    #[error("Invalid Database Format")]
    InvalidFormat,
}

// TODO: rename
pub trait DatabaseTrait<Ip> {
    fn get(&self, ip: Ip) -> Option<LookupInfo> {
        let crd = self.get_coordinate(ip)?;
        let loc = self.get_location(crd)?;

        Some(LookupInfo { crd, loc })
    }

    fn get_coordinate(&self, ip: Ip) -> Option<Coordinate>;
    fn get_location(&self, crd: Coordinate) -> Option<Location>;
}
