use std::{env, fs::File};

use ipgeo::GenericDatabase;

// before:
// Benchmark 1: target/release/examples/time
//   Time (mean ± σ):      2.256 s ±  0.016 s    [User: 2.157 s, System: 0.086 s]
//   Range (min … max):    2.237 s …  2.286 s    10 runs

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let GenericDatabase::Ipv4(db) = ipgeo::from_read(File::open(env::var("DB_PRELOADS")?)?)? else {
        panic!()
    };

    println!("{:#?}", db.get("1.1.1.1".parse().unwrap()));

    Ok(())
}
