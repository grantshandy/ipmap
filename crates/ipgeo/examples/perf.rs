use std::{
    fs::{self, File},
    net::Ipv4Addr,
    time::Instant,
};

use ipgeo::{ArchivedSingleDatabase, Database, Ipv4Database};
use memmap::Mmap;
use rkyv::rancor;

const TEST_COUNT: usize = 100_000;
const IP_MIN: u32 = Ipv4Addr::new(1, 0, 0, 0).to_bits();
const IP_MAX: u32 = Ipv4Addr::new(223, 0, 0, 0).to_bits();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    download().await?;

    test_mmapped()?;
    test_batch()?;

    Ok(())
}

fn test_mmapped() -> anyhow::Result<()> {
    let f = File::open("dbip-city-ipv4.ipgeodb")?;
    let bytes = unsafe { Mmap::map(&f)? };
    let db = unsafe { rkyv::access_unchecked::<ArchivedSingleDatabase<Ipv4Addr>>(&bytes) };

    test_db("mmaped", db);

    Ok(())
}

fn test_batch() -> anyhow::Result<()> {
    let db = rkyv::from_bytes::<Ipv4Database, rancor::Error>(&fs::read("dbip-city-ipv4.ipgeodb")?)?;
    test_db("batch", &db);
    Ok(())
}

fn test_db<T: Database<Ipv4Addr>>(label: &str, db: &T) {
    let start = Instant::now();

    println!("{:#?}", db.get(Ipv4Addr::new(1, 1, 1, 1)));

    for _ in 0..TEST_COUNT {
        let loc = db.get(fastrand::u32(IP_MIN..=IP_MAX).into());
        if let Some(loc) = loc {
            let _crd = loc.crd;
        }
    }

    let elapsed = start.elapsed();

    println!(
        "{label} took {}ms, average lookup at {}ns",
        elapsed.as_millis(),
        elapsed.as_nanos() / TEST_COUNT as u128
    );
}

async fn download() -> anyhow::Result<()> {
    let db = Ipv4Database::download(
        "http://0.0.0.0:8000/dbip-city/dbip-city-ipv4-num.csv.gz",
        true,
        |a, b| println!("{}%", (a as f64 / b as f64) * 100.0),
    )
    .await?;

    fs::write(
        "dbip-city-ipv4.ipgeodb",
        rkyv::to_bytes::<rkyv::rancor::Error>(&db)?,
    )?;

    Ok(())
}
