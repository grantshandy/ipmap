# IP-Geolocation Dataset Parser/Datastructures

Datasets can be downloaded at [`sapics/ip-location`](https://github.com/sapics/ip-location-db?tab=readme-ov-file#city) under the "City" section.
They are labeled `(ipdb/geolite2)-city-ipv(4/6)[-num].csv[.gz]` depending on the name/format, note that the `-num` ones are smaller/faster to parse.
All formats (including `.gz`) files are automatically detected and decompressed, except for `.7z` archives.

```rust
use std::{env, fs::File, net::IpAddr, error::Error};

use ipgeo::GeoDatabase;

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    let [_, db, ip, ..] = args.as_slice() else {
        eprintln!("Usage: ipgeo <db> <ip>");
        return Ok(());
    };

    let Ok(ip) = ip.parse::<IpAddr>() else {
        eprintln!("Invalid IP address: {ip}");
        return Ok(());
    };

    let db = GeoDatabase::from_read(&File::open(db)?)?;

    if db.is_ipv4() {
        println!("IPv4 database detected");
    } else {
        println!("IPv6 database detected");
    }

    if let Some((coord, loc)) = db.get(ip) {
        println!("{ip}:\n{coord:?}\n{loc:?}");
    } else {
        eprintln!("No location found for {ip}");
    };

    Ok(())
}
```

```sh
$ cargo r -rq -- ~/Documents/ipdbs/dbip-city-ipv4-num.csv.gz 220.126.94.12
IPv4 database detected
220.126.94.12:
Coordinate { lat: 37.3654, lng: 127.122 }
Location { city: Some("Seongnam Si Buljeong Ro"), region: Some("Gyeonggi Do"), country_code: "KR" }
```
