use compact_str::CompactString;
use maxminddb::{Reader, WithinItem};
use serde::Deserialize;
use treebitmap::IpLookupTable;

use crate::{
    Coordinate, Error, GenericIp,
    locations::{CountryCode, LocationIndices, LocationStore},
};

pub fn read<Ip: GenericIp, S: AsRef<[u8]>>(
    reader: Reader<S>,
    ips: &mut IpLookupTable<Ip, Coordinate>,
    locations: &mut LocationStore,
) -> Result<(), Error> {
    for res in reader
        .within::<CityFormat>(Ip::FULL_NETWORK)
        .map_err(Error::MaxMindDb)?
    {
        let WithinItem { ip_net, info } = res.map_err(Error::MaxMindDb)?;

        let ip = Ip::from_generic(ip_net.ip()).ok_or(Error::MalformedMaxMindDb)?;

        let coord = Coordinate {
            lat: info.latitude,
            lng: info.longitude,
        };

        locations.insert(coord, |strings| LocationIndices {
            city: strings.insert_str(info.city),
            region: strings.insert_str(info.state1),
            country_code: CountryCode::from(info.country_code),
        });

        ips.insert(ip, ip_net.prefix().into(), coord);
    }

    Ok(())
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
