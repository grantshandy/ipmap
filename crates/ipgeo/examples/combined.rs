use std::{env, fs::File};

use flate2::read::GzDecoder;
use ipgeo::{CombinedDatabase, Database};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);

    let ipv4 = File::open(args.next().unwrap())?;
    let ipv6 = File::open(args.next().unwrap())?;

    let db = CombinedDatabase::from_csv(GzDecoder::new(ipv4), GzDecoder::new(ipv6), true)?;

    println!("{:#?}", db.get(args.next().unwrap().parse()?));

    std::fs::write("./combined.ipgeo", postcard::to_allocvec(&db)?)?;

    Ok(())
}
