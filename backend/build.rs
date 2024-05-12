use std::{
    env,
    fs::{self, File},
    io::Write,
};

include!("src/db_types.rs");

use csv::ReaderBuilder;

fn main() {
    tauri_build::build();

    println!("cargo:rerun-if-env-changed=IPV4NUM_DB");

    let out_dir = env::var("OUT_DIR").unwrap();

    let database_str = if let Ok(ip_csv_path) = env::var("IPV4NUM_DB") {
        let mut db = GeoDb::new();

        ReaderBuilder::new()
            .has_headers(false)
            .from_reader(File::open(ip_csv_path).expect("Read IPV4NUM_DB database"))
            .deserialize::<CityRecordIpv4Num>()
            .for_each(|record| {
                let record = record.expect("deserialize record");
                db.insert(record.ip_range_start..=record.ip_range_end, record.into());
            });

        let mut db_path = format!("{out_dir}/encoded_db");

        #[cfg(windows)]
        {
            db_path = db_path.replace(r"/", r"\").replace(r"\", r"\\");
        }

        let mut db_file = File::create(&db_path).expect("open db");
        bincode::serialize_into(&mut db_file, &db).expect("serialize");

        db_file.flush().expect("flush db file");

        format!("Some(bincode::deserialize(include_bytes!(\"{db_path}\").as_slice()).expect(\"deserialize database\"))")
    } else {
        "None".to_string()
    };

    fs::write(
        format!("{out_dir}/database.rs"),
        format!("
            include!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/src/db_types.rs\"));
            lazy_static::lazy_static! {{ pub static ref DATABASE: Option<GeoDb> = {database_str}; }}
        "),
    )
    .expect("open database file");
}
