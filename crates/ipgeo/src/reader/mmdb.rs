use compact_str::CompactString;
use maxminddb::{LookupResult, PathElement, Reader, WithinOptions};
use serde::Deserialize;
use treebitmap::IpLookupTable;

use crate::{
    Coordinate, Error, GenericIp,
    coordinate::PackedCoordinate,
    locations::{CountryCode, LocationIndices, LocationStore},
};

pub fn read<Ip: GenericIp, S: AsRef<[u8]>>(
    reader: Reader<S>,
    ips: &mut IpLookupTable<Ip, PackedCoordinate>,
    locations: &mut LocationStore,
) -> Result<(), Error> {
    for res in reader
        .within(Ip::FULL_NETWORK, WithinOptions::default())
        .map_err(Error::MaxMindDb)?
    {
        let lookup = res.map_err(Error::MaxMindDb)?;
        let net = lookup.network().map_err(Error::MaxMindDb)?;
        let ip = Ip::from_generic(net.ip()).ok_or(Error::MalformedMaxMindDb)?;

        let coord: PackedCoordinate = Coordinate {
            lat: decode::<_, f32>(&lookup, "latitude")?,
            lng: decode::<_, f32>(&lookup, "longitude")?,
        }
        .into();

        locations.insert(coord, &|strings| {
            Ok(LocationIndices {
                city: strings.insert_str(decode::<_, CompactString>(&lookup, "city")?),
                region: strings.insert_str(decode::<_, CompactString>(&lookup, "state1")?),
                country_code: CountryCode::from(decode::<_, CompactString>(
                    &lookup,
                    "country_code",
                )?),
            })
        })?;

        ips.insert(ip, net.prefix().into(), coord);
    }

    Ok(())
}

fn decode<'a, L: AsRef<[u8]>, T: Deserialize<'a>>(
    lr: &LookupResult<'a, L>,
    p: &'static str,
) -> Result<T, Error> {
    lr.decode_path(&[PathElement::Key(p)])
        .map_err(Error::MaxMindDb)
        .and_then(|r| r.ok_or(Error::MalformedMaxMindDb))
}
