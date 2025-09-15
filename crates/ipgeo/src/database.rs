use std::{
    collections::HashMap,
    net::{AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr},
    num::{NonZero, ParseIntError},
    str::FromStr,
};

use compact_str::CompactString;
use heck::ToTitleCase;
use indexmap::IndexSet;
use ipnet::{Ipv4Subnets, Ipv6Subnets};
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use rustc_hash::FxBuildHasher;
use serde::{Deserialize, Serialize};
use treebitmap::IpLookupTable;

use crate::{
    DatabaseTrait, Result,
    location::{Coordinate, CountryCode, Location},
};

pub type Ipv4Database = Database<Ipv4Addr>;
pub type Ipv6Database = Database<Ipv6Addr>;

/// IpAddr -(map)-> PackedCoordinate -(locations)-> LocationIndices -(strings)-> Location
#[derive(PartialEq, Serialize, Deserialize)]
pub struct Database<Ip: GenericIp> {
    pub(crate) coordinates: IpLookupTable<Ip, Coordinate>,
    pub(crate) locations: HashMap<Coordinate, LocationIndices, FxBuildHasher>,
    pub(crate) strings: StringDict,
}

impl<Ip: GenericIp> DatabaseTrait<Ip> for Database<Ip> {
    fn get_coordinate(&self, ip: Ip) -> Option<Coordinate> {
        self.coordinates.longest_match(ip).map(|(_, _, c)| *c)
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.locations.get(&crd).map(|i| i.populate(&self.strings))
    }
}

/// A trait representing either Ipv4Addr or Ipv6Addr for the needs in the database.
pub trait GenericIp:
    FromStr<Err = AddrParseError> + From<Self::Bits> + treebitmap::Address + std::fmt::Debug + Ord
{
    type Bits: FromStr<Err = ParseIntError>;
    const FULL_NETWORK: IpNetwork;

    fn from_str_bytes(record: &[u8]) -> Result<Self> {
        Ok(CompactString::from_utf8(record)?.parse::<Self>()?)
    }

    fn from_num_bytes(record: &[u8]) -> Result<Self> {
        Ok(CompactString::from_utf8(record)?
            .parse::<Self::Bits>()?
            .into())
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

type StringDictKey = NonZero<u32>;

/// A compact database of strings that can store less than u32::MAX items.
#[derive(PartialEq, Eq, Default, Serialize, Deserialize)]
pub(crate) struct StringDict(IndexSet<CompactString, FxBuildHasher>);

impl StringDict {
    pub fn insert_str(&mut self, item: CompactString) -> Option<StringDictKey> {
        if item.is_empty() {
            return None;
        }

        self.insert_bytes(item.as_bytes())
    }

    pub fn insert_bytes(&mut self, item: &[u8]) -> Option<StringDictKey> {
        if item.is_empty() {
            return None;
        }

        let s = CompactString::from_utf8(item).ok()?.to_lowercase();
        let (idx, _) = self.0.insert_full(s);

        NonZero::new((idx + 1) as u32)
    }

    pub fn get(&self, idx: StringDictKey) -> Option<String> {
        self.0
            .get_index((idx.get() - 1) as usize)
            .map(|c| c.to_title_case())
    }
}

/// The city and region are stored as indexes into a string database.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocationIndices {
    pub(crate) city: Option<StringDictKey>,
    pub(crate) region: Option<StringDictKey>,
    pub(crate) country_code: CountryCode,
}

impl LocationIndices {
    fn populate(&self, strings: &StringDict) -> Location {
        Location {
            city: self.city.and_then(|i| strings.get(i)),
            region: self.region.and_then(|i| strings.get(i)),
            country_code: self.country_code.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_dict() {
        let mut s = StringDict::default();

        let city = "A Region".to_string();
        let region = "City Name Here".to_string();

        let city_idx = s.insert_str(city.clone().into()).unwrap();
        let region_idx = s.insert_bytes(region.as_bytes()).unwrap();

        assert_eq!(Some(city), s.get(city_idx));
        assert_eq!(Some(region), s.get(region_idx));
    }
}
