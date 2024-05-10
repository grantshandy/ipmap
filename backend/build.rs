use std::{
    env,
    fs::{self, File},
};

include!("src/db_types.rs");

use csv::ReaderBuilder;

fn main() {
    tauri_build::build();

    let Ok(Ok(ip_csv)) = env::var("IPV4NUM_DB").map(|v| File::open(v)) else {
        panic!("environment variable IPV4NUM_DB must be set");
    };

    let mut db = GeoDb::new();

    ReaderBuilder::new()
        .has_headers(false)
        .from_reader(ip_csv)
        .deserialize::<CityRecordIpv4Num>()
        .map(|record| {
            record.map(|record| {
                db.insert(record.ip_range_start..=record.ip_range_end, record.into());
            })
        })
        .collect::<Result<(), csv::Error>>()
        .unwrap();

    fs::write(
        format!("{}/encoded_db", env::var("OUT_DIR").unwrap()),
        bincode::serialize(&db).expect("serialize"),
    )
    .expect("write file")
}
