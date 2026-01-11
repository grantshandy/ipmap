use std::net::IpAddr;

use rkyv::{option::ArchivedOption, rend::NonZeroU32_le};

use crate::{
    Coordinate, Database, GenericIp, Location,
    database::{ArchivedCombinedDatabase, ArchivedSingleDatabase},
    locations::{
        ArchivedCoordinate, ArchivedCountryCode, ArchivedLocationIndices, ArchivedLocationStore,
        ArchivedStringDict, CountryCode,
    },
};

impl ArchivedCoordinate {
    fn as_bytes(&self) -> u64 {
        let mut out = [0; 8];
        let (one, two) = out.split_at_mut(4);
        one.copy_from_slice(self.lat.to_native().to_ne_bytes().as_slice());
        two.copy_from_slice(self.lng.to_native().to_ne_bytes().as_slice());
        u64::from_ne_bytes(out)
    }
}

impl From<&ArchivedCountryCode> for CountryCode {
    fn from(value: &ArchivedCountryCode) -> Self {
        Self(value.0.to_native())
    }
}

impl From<&ArchivedCoordinate> for Coordinate {
    fn from(value: &ArchivedCoordinate) -> Self {
        Self {
            lat: value.lat.to_native(),
            lng: value.lng.to_native(),
        }
    }
}

impl From<Coordinate> for ArchivedCoordinate {
    fn from(value: Coordinate) -> Self {
        Self {
            lat: value.lat.into(),
            lng: value.lng.into(),
        }
    }
}

impl From<ArchivedCoordinate> for Coordinate {
    fn from(value: ArchivedCoordinate) -> Self {
        Self {
            lat: value.lat.to_native(),
            lng: value.lng.to_native(),
        }
    }
}

impl PartialEq for ArchivedCoordinate {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}
impl Eq for ArchivedCoordinate {}

impl PartialOrd for ArchivedCoordinate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ArchivedCoordinate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_bytes().cmp(&other.as_bytes())
    }
}

impl std::hash::Hash for ArchivedCoordinate {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl ArchivedStringDict {
    pub fn get(&self, idx: NonZeroU32_le) -> Option<String> {
        self.0
            .get_index((idx.get() - 1) as usize)
            .map(|s| s.to_string())
    }
}

impl ArchivedLocationIndices {
    fn populate(&self, strings: &ArchivedStringDict) -> Location {
        let city = match self.city {
            ArchivedOption::Some(idx) => strings.get(idx),
            ArchivedOption::None => None,
        };

        let region = match self.region {
            ArchivedOption::Some(idx) => strings.get(idx),
            ArchivedOption::None => None,
        };

        Location {
            city,
            region,
            country_code: CountryCode::from(&self.country_code).to_string(),
        }
    }
}

impl ArchivedLocationStore {
    /// Get the location for an associated coordinate.
    pub fn get(&self, coord: Coordinate) -> Option<Location> {
        self.coordinates.get(&coord.into()).map(|i| {
            // UNWRAP: self.locations and self.coordinates are updated at the same time in Self::insert_location
            self.locations
                .get_index(i.to_native() as usize)
                .unwrap()
                .populate(&self.strings)
        })
    }
}

impl<Ip: GenericIp> Database<Ip> for ArchivedSingleDatabase<Ip> {
    fn get_coordinate(&self, ip: Ip) -> Option<Coordinate> {
        self.ips.longest_match(ip).map(|(_, _, c)| c.into())
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.locations.get(crd)
    }
}

impl Database<IpAddr> for ArchivedCombinedDatabase {
    fn get_coordinate(&self, ip: IpAddr) -> Option<Coordinate> {
        match ip {
            IpAddr::V4(ip) => self.ipv4.longest_match(ip).map(|(_, _, c)| c.into()),
            IpAddr::V6(ip) => self.ipv6.longest_match(ip).map(|(_, _, c)| c.into()),
        }
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.locations.get(crd)
    }
}
