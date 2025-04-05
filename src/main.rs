use std::{env, fs::File, net::IpAddr};

use ipmap::ipgeo::GeoDatabase;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(env::args().nth(1).expect("pass in the db"))?;

    let db = GeoDatabase::from_read(&file)?;

    let ip = env::args()
        .nth(2)
        .expect("pass in the IP")
        .parse::<IpAddr>()?;

    std::thread::sleep(std::time::Duration::from_secs(40));

    println!("{:?}", db.get(ip));

    Ok(())
}
