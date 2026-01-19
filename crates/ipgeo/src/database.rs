use std::{
    io::Read,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use treebitmap::IpLookupTable;

use crate::{
    Coordinate, Database, Error, GenericIp,
    coordinate::PackedCoordinate,
    locations::{Location, LocationStore},
    reader,
};

/// A database that stores IPv4 addresses.
pub type Ipv4Database = SingleDatabase<Ipv4Addr>;

/// A database that stores IPv6 addresses.
pub type Ipv6Database = SingleDatabase<Ipv6Addr>;

/// A database for a single address type.
#[derive(
    PartialEq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct SingleDatabase<Ip: GenericIp> {
    pub(crate) ips: IpLookupTable<Ip, PackedCoordinate>,
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
        self.ips.longest_match(ip).map(|(_, _, c)| c.into())
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.locations.get(&(crd.into()))
    }
}

/// A database built from both an IPv4 and IPv6 CSV file.
#[derive(PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct CombinedDatabase {
    pub(crate) ipv4: IpLookupTable<Ipv4Addr, PackedCoordinate>,
    pub(crate) ipv6: IpLookupTable<Ipv6Addr, PackedCoordinate>,
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
            IpAddr::V4(ip) => self.ipv4.longest_match(ip).map(|(_, _, c)| c.into()),
            IpAddr::V6(ip) => self.ipv6.longest_match(ip).map(|(_, _, c)| c.into()),
        }
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        self.locations.get(&(crd.into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Coordinate, Database, Ipv4Database, Ipv6Database, Location, LookupInfo};
    use std::{
        error,
        net::{Ipv4Addr, Ipv6Addr},
    };

    fn info() -> LookupInfo {
        LookupInfo {
            crd: Coordinate {
                lat: 23.1317,
                lng: 113.266,
            },
            loc: Location {
                city: Some("Guangzhou".to_string()),
                region: Some("Guangdong".to_string()),
                country_code: "CN".to_string(),
            },
        }
    }

    fn format_csv_line(lower: String, higher: String, info: &LookupInfo) -> String {
        format!(
            "{lower},{higher},{},{},,{},,{},{},",
            info.loc.country_code.clone(),
            info.loc.region.clone().unwrap_or_default(),
            info.loc.city.clone().unwrap_or_default(),
            info.crd.lat,
            info.crd.lng
        )
    }

    #[test]
    fn parse_ipv4() -> Result<(), Box<dyn error::Error>> {
        let info = info();

        let ipv4_lower: Ipv4Addr = "1.0.8.0".parse()?;
        let ipv4_higher: Ipv4Addr = "1.0.15.255".parse()?;
        let ipv4_valid: Ipv4Addr = "1.0.9.80".parse()?;
        let ipv4_invalid: Ipv4Addr = "19.0.9.80".parse()?;

        let ipv4_db = Ipv4Database::from_csv(
            format_csv_line(ipv4_lower.to_string(), ipv4_higher.to_string(), &info).as_bytes(),
            false,
        )?;

        let result = ipv4_db.get(ipv4_valid);
        assert!(
            result.is_some() && result.as_ref().unwrap().approx_eq(&info),
            "IPv4 DB: expected {:?}, got {:?}",
            info,
            result
        );
        assert_eq!(None, ipv4_db.get(ipv4_invalid));

        let ipv4_num_db = Ipv4Database::from_csv(
            format_csv_line(
                ipv4_lower.to_bits().to_string(),
                ipv4_higher.to_bits().to_string(),
                &info,
            )
            .as_bytes(),
            true,
        )?;

        let result = ipv4_num_db.get(ipv4_valid);
        assert!(
            result.is_some() && result.as_ref().unwrap().approx_eq(&info),
            "IPv4 Num DB: expected {:?}, got {:?}",
            info,
            result
        );
        assert_eq!(None, ipv4_num_db.get(ipv4_invalid));

        Ok(())
    }

    #[test]
    fn parse_ipv6() -> Result<(), Box<dyn error::Error>> {
        let info = info();

        let ipv6_lower: Ipv6Addr = "2001:2::".parse()?;
        let ipv6_higher: Ipv6Addr = "2001:2::ffff:ffff:ffff:ffff:ffff".parse()?;
        let ipv6_valid: Ipv6Addr = "2001:2::9".parse()?;
        let ipv6_invalid: Ipv6Addr = "3001:2::9".parse()?;

        let ipv6_db = Ipv6Database::from_csv(
            format_csv_line(ipv6_lower.to_string(), ipv6_higher.to_string(), &info).as_bytes(),
            false,
        )?;

        let result = ipv6_db.get(ipv6_valid);
        assert!(
            result.is_some() && result.as_ref().unwrap().approx_eq(&info),
            "IPv6 DB: expected {:?}, got {:?}",
            info,
            result
        );
        assert_eq!(None, ipv6_db.get(ipv6_invalid));

        let ipv6_num_db = Ipv6Database::from_csv(
            format_csv_line(
                ipv6_lower.to_bits().to_string(),
                ipv6_higher.to_bits().to_string(),
                &info,
            )
            .as_bytes(),
            true,
        )?;

        let result = ipv6_num_db.get(ipv6_valid);
        assert!(
            result.is_some() && result.as_ref().unwrap().approx_eq(&info),
            "IPv6 Num DB: expected {:?}, got {:?}",
            info,
            result
        );
        assert_eq!(None, ipv6_num_db.get(ipv6_invalid));

        Ok(())
    }
}
