use std::{
    io::Read,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use serde::{Deserialize, Serialize};
use treebitmap::IpLookupTable;

use crate::{
    Database, Error, GenericIp,
    locations::{Coordinate, Location, LocationStore},
    reader,
};

/// A database that stores IPv4 addresses.
pub type Ipv4Database = SingleDatabase<Ipv4Addr>;

/// A database that stores IPv6 addresses.
pub type Ipv6Database = SingleDatabase<Ipv6Addr>;

/// A database for a single address type.
#[derive(PartialEq, Serialize, Deserialize)]
pub struct SingleDatabase<Ip: GenericIp> {
    pub(crate) ips: IpLookupTable<Ip, Coordinate>,
    pub(crate) locations: LocationStore,
}

impl<Ip: GenericIp> SingleDatabase<Ip> {
    pub fn from_csv(read: impl Read, is_num: bool) -> Result<Self, Error> {
        let mut ips = IpLookupTable::new();
        let mut locations = LocationStore::default();

        reader::csv::read(read, is_num, &mut ips, &mut locations)?;

        Ok(Self { ips, locations })
    }

    pub fn from_mmdb<S: AsRef<[u8]>>(reader: maxminddb::Reader<S>) -> Result<Self, Error> {
        let mut ips = IpLookupTable::new();
        let mut locations = LocationStore::default();

        reader::mmdb::read(reader, &mut ips, &mut locations)?;

        Ok(Self { ips, locations })
    }
}

impl<Ip: GenericIp> Database<Ip> for SingleDatabase<Ip> {
    fn get_coordinate(&self, ip: Ip) -> Option<Coordinate> {
        self.ips.longest_match(ip).map(|(_, _, c)| *c)
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.locations.get(crd)
    }
}

/// A database built from both an IPv4 and IPv6 CSV file.
#[derive(PartialEq, Serialize, Deserialize)]
pub struct CombinedDatabase {
    pub(crate) ipv4: IpLookupTable<Ipv4Addr, Coordinate>,
    pub(crate) ipv6: IpLookupTable<Ipv6Addr, Coordinate>,
    pub(crate) locations: LocationStore,
}

impl CombinedDatabase {
    pub fn from_csv(ipv4_csv: impl Read, ipv6_csv: impl Read, is_num: bool) -> Result<Self, Error> {
        let mut ipv4 = IpLookupTable::new();
        let mut ipv6 = IpLookupTable::new();
        let mut locations = LocationStore::default();

        reader::csv::read(ipv4_csv, is_num, &mut ipv4, &mut locations)?;
        reader::csv::read(ipv6_csv, is_num, &mut ipv6, &mut locations)?;

        Ok(Self {
            ipv4,
            ipv6,
            locations,
        })
    }
}

impl Database<IpAddr> for CombinedDatabase {
    fn get_coordinate(&self, ip: IpAddr) -> Option<Coordinate> {
        match ip {
            IpAddr::V4(ip) => self.ipv4.longest_match(ip).map(|(_, _, c)| *c),
            IpAddr::V6(ip) => self.ipv6.longest_match(ip).map(|(_, _, c)| *c),
        }
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.locations.get(crd)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Coordinate, Database, Ipv4Database, Ipv6Database, Location, LookupInfo};
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn parse() {
        let info = LookupInfo {
            crd: Coordinate {
                lat: 23.1317,
                lng: 113.266,
            },
            loc: Location {
                city: Some("Guangzhou".to_string()),
                region: Some("Guangdong".to_string()),
                country_code: "CN".to_string(),
            },
        };

        let format_csv_line = |lower: String, higher: String| {
            format!(
                "{lower},{higher},{},{},,{},,{},{},",
                info.loc.country_code.clone(),
                info.loc.region.clone().unwrap_or_default(),
                info.loc.city.clone().unwrap_or_default(),
                info.crd.lat,
                info.crd.lng
            )
        };

        let ipv4_lower: Ipv4Addr = "1.0.8.0".parse().unwrap();
        let ipv4_higher: Ipv4Addr = "1.0.15.255".parse().unwrap();
        let ipv4_valid: Ipv4Addr = "1.0.9.80".parse().unwrap();
        let ipv4_invalid: Ipv4Addr = "19.0.9.80".parse().unwrap();

        let ipv4_db = Ipv4Database::from_csv(
            format_csv_line(ipv4_lower.to_string(), ipv4_higher.to_string()).as_bytes(),
            false,
        )
        .unwrap();

        assert_eq!(Some(info.clone()), ipv4_db.get(ipv4_valid));
        assert_eq!(None, ipv4_db.get(ipv4_invalid));

        let ipv4_num_db = Ipv4Database::from_csv(
            format_csv_line(
                ipv4_lower.to_bits().to_string(),
                ipv4_higher.to_bits().to_string(),
            )
            .as_bytes(),
            true,
        )
        .unwrap();

        assert_eq!(Some(info.clone()), ipv4_num_db.get(ipv4_valid));
        assert_eq!(None, ipv4_num_db.get(ipv4_invalid));

        let ipv6_lower: Ipv6Addr = "2001:2::".parse().unwrap();
        let ipv6_higher: Ipv6Addr = "2001:2::ffff:ffff:ffff:ffff:ffff".parse().unwrap();
        let ipv6_valid: Ipv6Addr = "2001:2::9".parse().unwrap();
        let ipv6_invalid: Ipv6Addr = "3001:2::9".parse().unwrap();

        let ipv6_db = Ipv6Database::from_csv(
            format_csv_line(ipv6_lower.to_string(), ipv6_higher.to_string()).as_bytes(),
            false,
        )
        .unwrap();

        assert_eq!(Some(info.clone()), ipv6_db.get(ipv6_valid));
        assert_eq!(None, ipv6_db.get(ipv6_invalid));

        let ipv6_num_db = Ipv6Database::from_csv(
            format_csv_line(
                ipv6_lower.to_bits().to_string(),
                ipv6_higher.to_bits().to_string(),
            )
            .as_bytes(),
            true,
        )
        .unwrap();

        assert_eq!(Some(info.clone()), ipv6_num_db.get(ipv6_valid));
        assert_eq!(None, ipv6_num_db.get(ipv6_invalid));
    }
}
