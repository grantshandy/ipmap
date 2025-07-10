use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
    time::Instant,
};

use ipgeo::{Ipv4Database, Ipv6Database};

mod shared;

pub struct PreloadedDatabases {
    pub ipv4: Vec<(PathBuf, Arc<Ipv4Database>)>,
    pub ipv6: Vec<(PathBuf, Arc<Ipv6Database>)>,
}

const DB_PRELOADS_BIN: &[u8] = include_bytes!(env!("DB_PRELOADS_BIN"));

pub static DB_PRELOADS: LazyLock<PreloadedDatabases> = LazyLock::new(|| {
    tracing::info!("Loading internal databases");

    let start = Instant::now();
    let (ipv4, ipv6) = postcard::from_bytes::<shared::DiskDatabases>(DB_PRELOADS_BIN).unwrap();
    tracing::info!(
        "Loading internal databases took {}ms",
        start.elapsed().as_millis()
    );

    PreloadedDatabases {
        ipv4: ipv4
            .into_iter()
            .map(|(path, db)| (path, Arc::new(db)))
            .collect(),
        ipv6: ipv6
            .into_iter()
            .map(|(path, db)| (path, Arc::new(db)))
            .collect(),
    }
});

pub fn load_builtins(state: &super::DbState) {
    for (path, db) in &DB_PRELOADS.ipv4 {
        state.ipv4_db.insert_arc(path, db.clone(), true);
    }

    for (path, db) in &DB_PRELOADS.ipv6 {
        state.ipv6_db.insert_arc(path, db.clone(), true);
    }
}
