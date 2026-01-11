# IP-Geolocation Dataset Parser/Datastructures

Datasets can be downloaded at [`sapics/ip-location`](https://github.com/sapics/ip-location-db?tab=readme-ov-file#city) under the "City" section.
They are labeled `(ipdb/geolite2)-city-ipv(4/6)[-num].csv[.gz]` depending on the name/format, note that the `-num` ones are smaller/faster to parse. It also supports maxmindb files in the same format.
All formats (including `.gz`) files are automatically detected and decompressed, except for `.7z` archives.

<!--```rust,no_run,no_test
use std::{env, net::IpAddr, path::PathBuf};

use ipgeo::DatabaseTrait;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);

    let path = PathBuf::from(args.next().unwrap());
    let ip: IpAddr = args.next().unwrap().parse()?;

    let db = ipgeo::detect(&path)?;

    println!("{:#?}", db.get(ip));

    Ok(())
}
```-->

```sh
$ cargo r -rq -- ~/Documents/ipdbs/dbip-city-ipv4-num.csv.gz 1.1.1.1
Some(
    LookupInfo {
        crd: Coordinate {
            lat: -33.8688,
            lng: 151.209,
        },
        loc: Location {
            city: Some(
                "Sydney",
            ),
            region: Some(
                "New South Wales",
            ),
            country_code: "AU",
        },
    },
)
```

## Download Notes
Parallel: Downloaded 708.95 MB, decompressing/parsing at 100.03 MB/s in 7 seconds
