use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

#[path = "src/db_types.rs"]
mod db_types;

fn main() {
    tauri_build::build();

    println!("cargo:rerun-if-env-changed=IPV4NUM_DB");

    let out_dir = env::var("OUT_DIR").unwrap();

    let database_str = if let Ok(ip_csv_path) = env::var("IPV4NUM_DB") {
        let attribution =
            env::var("IPV4NUM_DB_ATTRIBUTION").expect("IPV4NUM_DB_ATTRIBUTION must be set.");

        let (db, locations) = db_types::read_csv(File::open(&ip_csv_path).expect("Read IPV4NUM_DB database"))
            .expect("read csv");

        let mut db_path = format!("{out_dir}/encoded_db");
        #[cfg(windows)]
        {
            db_path = db_path.replace(r"/", r"\").replace(r"\", r"\\");
        }
        // println!("cargo:rerun-if-changed={db_path}");

        let mut db_file = File::create(&db_path).expect("open db");
        bincode::serialize_into(&mut db_file, &db).expect("serialize");
        db_file.flush().expect("flush db file");

        let build_time = db_types::build_time();
        let db_name = Path::new(&ip_csv_path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        format!("Some(Database {{
            db: bincode::deserialize(include_bytes!(\"{db_path}\").as_slice()).expect(\"deserialize database\"),
            info: Info {{
                name: \"{db_name} (built in)\".to_string(),
                path: None,
                build_time: \"{build_time}\".to_string(),
                attribution_text: Some(\"{attribution}\".to_string()),
                locations: {locations}
            }}
        }})")
    } else {
        "None".to_string()
    };

    fs::write(
        format!("{out_dir}/database.rs"),
        format!(
            "
            include!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/src/db_types.rs\"));
            lazy_static::lazy_static! {{ pub static ref DATABASE: Option<Database> = {database_str}; }}
        "
        ),
    )
    .expect("open database file");
}
