use std::{env, fs::File, net::Ipv4Addr};

use ipgeo::GenericDatabase;

// 100% monomorphized:
// 848K	target/release/examples/time
// Benchmark 1: target/release/examples/time ~/Downloads/dbip-city-ipv4.csv.gz
// Time (mean ± σ):      2.600 s ±  0.017 s    [User: 2.517 s, System: 0.070 s]
// Range (min … max):    2.580 s …  2.630 s    10 runs

// gzip read is dyn dispatch
// 836K	target/release/examples/time
// Benchmark 1: target/release/examples/time ~/Downloads/dbip-city-ipv4.csv.gz
// Time (mean ± σ):      2.607 s ±  0.011 s    [User: 2.516 s, System: 0.078 s]
// Range (min … max):    2.591 s …  2.623 s    10 runs

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).unwrap();

    let GenericDatabase::Ipv4(db) = ipgeo::from_read(File::open(path)?)? else {
        panic!("wrong type");
    };

    let ip: Ipv4Addr = "1.1.1.1".parse().unwrap();

    for _ in 0..=500 {
        db.get(ip).unwrap();
    }

    Ok(())
}
