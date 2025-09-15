use std::{
    io::Read,
    path::PathBuf,
    sync::{Arc, LazyLock},
    time::Instant,
};

use flate2::bufread::GzDecoder;
use ipgeo::{Ipv4Database, Ipv6Database};

mod shared;

#[derive(Default)]
pub struct PreloadedDatabases {
    pub ipv4: Vec<(PathBuf, Arc<Ipv4Database>)>,
    pub ipv6: Vec<(PathBuf, Arc<Ipv6Database>)>,
}

const DB_PRELOADS_BIN: &[u8] = include_bytes!(env!("DB_PRELOADS_BIN"));

pub static DB_PRELOADS: LazyLock<PreloadedDatabases> = LazyLock::new(|| {
    tracing::info!("Loading internal databases...");

    let start = Instant::now();

    let dbs = match uncompress_databases() {
        Ok(dbs) => dbs,
        Err(err) => {
            tracing::error!("Failed to load internal databases: {err:?}");
            return PreloadedDatabases::default();
        }
    };

    tracing::info!(
        "Loading internal databases took {}ms",
        start.elapsed().as_millis()
    );

    dbs
});

pub fn load_builtins(state: &super::DbState) {
    for (path, db) in &DB_PRELOADS.ipv4 {
        state.ipv4_db.insert_arc(path, db.clone(), true);
    }

    for (path, db) in &DB_PRELOADS.ipv6 {
        state.ipv6_db.insert_arc(path, db.clone(), true);
    }
}

fn uncompress_databases() -> Result<PreloadedDatabases, Box<dyn std::error::Error>> {
    // Generally observed to be around 4x
    let mut uncompressed = Vec::with_capacity(DB_PRELOADS_BIN.len() * 4);
    GzDecoder::new(DB_PRELOADS_BIN).read_to_end(&mut uncompressed)?;

    let (ipv4, ipv6) = postcard::from_bytes::<shared::DiskDatabases>(&uncompressed)?;

    drop(uncompressed);

    Ok(PreloadedDatabases {
        ipv4: ipv4
            .into_iter()
            .map(|(path, db)| (path, Arc::new(db)))
            .collect(),
        ipv6: ipv6
            .into_iter()
            .map(|(path, db)| (path, Arc::new(db)))
            .collect(),
    })
}
