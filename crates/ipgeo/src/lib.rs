#![doc = include_str!("../README.md")]

use std::{
    net::{AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr},
    num::{ParseFloatError, ParseIntError},
    str::{FromStr, Utf8Error},
};

use ipnet::{Ipv4Subnets, Ipv6Subnets};
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};

mod coordinate;
mod database;
mod detect;
mod reader;

pub mod locations;
pub(crate) mod rkyv_impl;

#[cfg(feature = "download")]
pub mod download;

pub use coordinate::Coordinate;
pub use database::{
    ArchivedCombinedDatabase, ArchivedSingleDatabase, CombinedDatabase, Ipv4Database, Ipv6Database,
    SingleDatabase,
};
pub use detect::{ArchivedGenericDatabase, GenericDatabase, detect};
pub use locations::{Location, LookupInfo};
pub use treebitmap;

/// A generic way of addressing a [`CombinedDatabase`], [`SingleDatabase`], or [`GenericDatabase`].
pub trait Database<Ip> {
    /// Get a [`Coordinate`]/[`Location`] pair for a given ip address.
    fn get(&self, ip: Ip) -> Option<LookupInfo> {
        let crd = self.get_coordinate(ip)?;
        let loc = self.get_location(crd)?;

        Some(LookupInfo { crd, loc })
    }

    fn get_coordinate(&self, ip: Ip) -> Option<Coordinate>;
    fn get_location(&self, crd: Coordinate) -> Option<Location>;
}

/// A trait representing either an `Ipv4Addr` or `Ipv6Addr` for the needs in the database.
#[doc(hidden)]
pub trait GenericIp:
    FromStr<Err = AddrParseError>
    + From<Self::Bits>
    + treebitmap::Address
    + std::fmt::Debug
    + Ord
    + Send
    + Sync
    + 'static
{
    type Bits: FromStr<Err = ParseIntError>;
    const FULL_NETWORK: IpNetwork;

    fn from_str_bytes(record: &[u8]) -> Result<Self, Error> {
        Ok(str::from_utf8(record)?.parse::<Self>()?)
    }

    fn from_num_bytes(record: &[u8]) -> Result<Self, Error> {
        Ok(str::from_utf8(record)?.parse::<Self::Bits>()?.into())
    }

    fn from_generic(ip: IpAddr) -> Option<Self>;
    fn range_subnets(start: Self, end: Self) -> impl Iterator<Item = (Self, u32)>;
}

impl GenericIp for Ipv4Addr {
    type Bits = u32;
    const FULL_NETWORK: IpNetwork =
        IpNetwork::V4(Ipv4Network::new_checked(Ipv4Addr::UNSPECIFIED, 0).unwrap());

    fn from_generic(ip: IpAddr) -> Option<Self> {
        match ip {
            IpAddr::V4(ip) => Some(ip),
            IpAddr::V6(_) => None,
        }
    }

    fn range_subnets(start: Self, end: Self) -> impl Iterator<Item = (Self, u32)> {
        Ipv4Subnets::new(start, end, 0).map(|net| (net.addr(), net.prefix_len().into()))
    }
}

impl GenericIp for Ipv6Addr {
    type Bits = u128;
    const FULL_NETWORK: IpNetwork =
        IpNetwork::V6(Ipv6Network::new_checked(Ipv6Addr::UNSPECIFIED, 0).unwrap());

    fn from_generic(ip: IpAddr) -> Option<Self> {
        match ip {
            IpAddr::V4(_) => None,
            IpAddr::V6(ip) => Some(ip),
        }
    }

    fn range_subnets(start: Self, end: Self) -> impl Iterator<Item = (Self, u32)> {
        Ipv6Subnets::new(start, end, 0).map(|net| (net.addr(), net.prefix_len().into()))
    }
}

/// Any error that may occur while parsing a database.
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
    Utf8(#[from] Utf8Error),
    #[error("Parsing IP integer: {0}")]
    IpNumParse(#[from] ParseIntError),
    #[error("Parsing IP: {0}")]
    IpStrParse(#[from] AddrParseError),
    /// Indexes into the string database are stored in u32s to save space.
    ///
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
