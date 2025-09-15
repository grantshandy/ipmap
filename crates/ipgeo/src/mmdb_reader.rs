use compact_str::CompactString;
use maxminddb::{Reader, WithinItem};
use serde::Deserialize;

use crate::{
    Coordinate, Database, Error, GenericIp, Result, database::LocationIndices,
    location::CountryCode,
};

impl<Ip: GenericIp> Database<Ip> {
    pub fn from_mmdb<S: AsRef<[u8]>>(reader: Reader<S>) -> Result<Self> {
        let mut db = Self::empty();

        for res in reader
            .within::<CityFormat>(Ip::FULL_NETWORK)
            .map_err(Error::MaxMindDb)?
        {
            let WithinItem { ip_net, info } = res.map_err(Error::MaxMindDb)?;

            let coord = Coordinate {
                lat: info.latitude,
                lng: info.longitude,
            };

            db.insert_location(coord, |strings| LocationIndices {
                city: strings.insert_str(info.city),
                region: strings.insert_str(info.state1),
                country_code: CountryCode::from(info.country_code),
            });

            db.ips.insert(
                Ip::from_generic(ip_net.ip()).ok_or(Error::MalformedMaxMindDb)?,
                ip_net.prefix().into(),
                coord,
            );
        }

        Ok(db)
    }
}

/// https://github.com/sapics/ip-location-db/tree/main/dbip-city-mmdb#mmdb-format
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct CityFormat {
    country_code: CompactString,
    city: CompactString,
    state1: CompactString,
    state2: CompactString,
    postcode: CompactString,
    latitude: f32,
    longitude: f32,
    timezone: CompactString,
}
