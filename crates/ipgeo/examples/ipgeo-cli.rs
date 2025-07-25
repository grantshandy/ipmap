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
