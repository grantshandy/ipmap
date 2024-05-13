use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

use time::OffsetDateTime;

#[path = "src/db_types.rs"]
mod db_types;

fn main() {
    tauri_build::build();

    println!("cargo:rerun-if-env-changed=IPV4NUM_DB");

    let out_dir = env::var("OUT_DIR").unwrap();

    let (database_str, database_info) = if let Ok(ip_csv_path) = env::var("IPV4NUM_DB") {
        let attribution = env::var("IPV4NUM_DB_ATTRIBUTION").expect("IPV4NUM_DB_ATTRIBUTION must be set.");

        let db = db_types::read_csv(File::open(&ip_csv_path).expect("Read IPV4NUM_DB database"))
            .expect("read csv");

        let mut db_path = format!("{out_dir}/encoded_db");
        #[cfg(windows)]
        {
            db_path = db_path.replace(r"/", r"\").replace(r"\", r"\\");
        }
        println!("cargo:rerun-if-changed={db_path}");

        let mut db_file = File::create(&db_path).expect("open db");
        bincode::serialize_into(&mut db_file, &db).expect("serialize");
        db_file.flush().expect("flush db file");

        let db_name = Path::new(&ip_csv_path)
            .file_stem()
            .expect("get ipdb csv filename")
            .to_string_lossy();
        let build_time = OffsetDateTime::now_utc().to_string();

        (
            format!("Some(bincode::deserialize(include_bytes!(\"{db_path}\").as_slice()).expect(\"deserialize database\"))"),
            format!("Some(DatabaseInfo {{ filename: \"{db_name}\", built: \"{build_time}\", attribution: \"{attribution}\" }})")
        )
    } else {
        ("None".to_string(), "None".to_string())
    };

    fs::write(
        format!("{out_dir}/database.rs"),
        format!(
            "
            include!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/src/db_types.rs\"));
            pub const INFO: Option<DatabaseInfo> = {database_info};
            lazy_static::lazy_static! {{ pub static ref DATABASE: Option<GeoDb> = {database_str}; }}
        "
        ),
    )
    .expect("open database file");
}
