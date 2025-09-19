use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::Path,
};

use compact_str::CompactString;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};

use crate::{Coordinate, Database, Error, Location, SingleDatabase};

/// Automatically detect the format of the database and read it.
///
/// Accepts ip-location "city" `*.mmdb`, `*-num.csv` and `*-num.csv.gz` files.
pub fn detect(path: &Path) -> Result<GenericDatabase, Error> {
    match DatabaseKind::detect(path)? {
        DatabaseKind::Csv {
            reader,
            is_num,
            is_ipv6,
        } => match is_ipv6 {
            true => SingleDatabase::from_csv(reader, is_num).map(GenericDatabase::Ipv6),
            false => SingleDatabase::from_csv(reader, is_num).map(GenericDatabase::Ipv4),
        },
        DatabaseKind::Maxminddb { reader } => match reader.metadata.ip_version {
            4 => SingleDatabase::from_mmdb(reader).map(GenericDatabase::Ipv4),
            6 => SingleDatabase::from_mmdb(reader).map(GenericDatabase::Ipv6),
            _ => Err(Error::MalformedMaxMindDb),
        },
    }
}

/// A generic [`SingleDatabase`].
#[derive(PartialEq, Serialize, Deserialize)]
pub enum GenericDatabase {
    Ipv4(SingleDatabase<Ipv4Addr>),
    Ipv6(SingleDatabase<Ipv6Addr>),
}

impl GenericDatabase {
    pub fn is_ipv4(&self) -> bool {
        matches!(self, Self::Ipv4(_))
    }

    pub fn is_ipv6(&self) -> bool {
        matches!(self, Self::Ipv6(_))
    }
}

impl Database<IpAddr> for GenericDatabase {
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

enum DatabaseKind {
    Csv {
        reader: Box<dyn Read>,
        is_num: bool,
        is_ipv6: bool,
    },
    Maxminddb {
        reader: maxminddb::Reader<Vec<u8>>,
    },
}

impl DatabaseKind {
    fn detect(path: &Path) -> Result<Self, Error> {
        match path.extension().and_then(|s| s.to_str()) {
            Some("mmdb") => Self::parse_mmdb(path),
            _ => Self::parse_csv(File::open(path)?),
        }
    }

    fn parse_mmdb(path: &Path) -> Result<Self, Error> {
        match maxminddb::Reader::open_readfile(path) {
            Ok(reader) => Ok(Self::Maxminddb { reader }),
            Err(e) => Err(Error::MaxMindDb(e)),
        }
    }

    fn parse_csv(mut f: File) -> Result<Self, Error> {
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
