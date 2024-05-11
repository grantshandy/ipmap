use std::{
    env,
    fs::{self, File}
};

mod db_types {
    include!("src/db_types.rs");
}

use csv::ReaderBuilder;

fn main() {
    tauri_build::build();

    println!("cargo:rerun-if-env-changed=IPV4NUM_DB");

    let out_dir = env::var("OUT_DIR").unwrap();

    let database_str = if let Ok(ip_csv_path) = env::var("IPV4NUM_DB") {
        let ip_csv = File::open(ip_csv_path).expect("Read IPV4NUM_DB database");
        let mut db = db_types::GeoDb::new();

        ReaderBuilder::new()
            .has_headers(false)
            .from_reader(ip_csv)
            .deserialize::<db_types::CityRecordIpv4Num>()
            .for_each(|record| {
                let record = record.expect("deserialize record");
                db.insert(record.ip_range_start..=record.ip_range_end, record.into());
            });

        let mut db_path = format!("{out_dir}/encoded_db");

        #[cfg(windows)]
        {
            db_path = db_path.replace(r"/", r"\").replace(r"\", r"\\");
        }

        fs::write(
            &db_path,
            postcard::to_stdvec(&db).expect("serialize database"),
        )
        .expect("write database to disk");

        format!("Some(postcard::from_bytes(&include_bytes!(\"{db_path}\")[..]).expect(\"deserialize database\"))")
    } else {
        "None".to_string()
    };

    fs::write(
        format!("{out_dir}/database.rs"),
        format!(r#"
            pub mod db_types {{ include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/db_types.rs")); }}
            lazy_static::lazy_static! {{ pub static ref DATABASE: Option<db_types::GeoDb> = {database_str}; }}
        "#)
    ).expect("open database file");
}
