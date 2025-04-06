use std::{env, error::Error, fs::File, net::IpAddr};

use ipmap::GeoDatabase;

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
