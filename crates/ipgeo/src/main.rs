use std::{env, fs::File, net::IpAddr};

use ipmap::GeoDatabase;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(env::args().nth(1).expect("pass in the db"))?;

    let db = GeoDatabase::from_read(&file)?;

    let ip = env::args()
        .nth(2)
        .expect("pass in the IP")
        .parse::<IpAddr>()?;

    let Some((coord, loc)) = db.get(ip) else {
        eprintln!("No location found for {ip}");
        return Ok(());
    };

    println!("{ip}:\n{coord:?}\n{loc:?}");

    Ok(())
}
