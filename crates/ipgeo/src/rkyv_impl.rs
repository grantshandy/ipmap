use std::net::IpAddr;

use rkyv::{
    option::ArchivedOption,
    rend::{NonZeroU32_le, u16_le},
};

use crate::{
    Coordinate, Database, GenericIp, Location,
    coordinate::{ArchivedPackedCoordinate, PackedCoordinate},
    database::{ArchivedCombinedDatabase, ArchivedSingleDatabase},
    locations::{
        ArchivedCountryCode, ArchivedLocationIndices, ArchivedLocationStore, ArchivedStringDict,
        CountryCode,
    },
};

impl From<&ArchivedCountryCode> for CountryCode {
    fn from(value: &ArchivedCountryCode) -> Self {
        Self(value.0.to_native())
    }
}

impl From<&ArchivedPackedCoordinate> for Coordinate {
    fn from(value: &ArchivedPackedCoordinate) -> Self {
        (&PackedCoordinate {
            lat_u: value.lat_u.to_native(),
            lng_u: value.lng_u.to_native(),
        })
            .into()
    }
}

impl From<PackedCoordinate> for ArchivedPackedCoordinate {
    fn from(value: PackedCoordinate) -> Self {
        Self {
            lat_u: u16_le::from_native(value.lat_u),
            lng_u: u16_le::from_native(value.lng_u),
        }
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
        Location {
            city: match self.city {
                ArchivedOption::Some(idx) => strings.get(idx),
                ArchivedOption::None => None,
            },
            region: match self.region {
                ArchivedOption::Some(idx) => strings.get(idx),
                ArchivedOption::None => None,
            },
            country_code: CountryCode::from(&self.country_code).to_string(),
        }
    }
}

impl ArchivedLocationStore {
    /// Get the location for an associated coordinate.
    pub fn get(&self, coord: ArchivedPackedCoordinate) -> Option<Location> {
        self.coordinates.get(&coord).map(|i| {
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
        let crd: PackedCoordinate = crd.into();
        self.locations.get(crd.into())
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
        let crd: PackedCoordinate = crd.into();
        self.locations.get(crd.into())
    }
}
