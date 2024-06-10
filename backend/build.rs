use std::{env, fs};

#[path = "src/db_types.rs"]
mod db_types;

use db_types::Database;

fn main() {
    tauri_build::build();

    // println!("cargo:rerun-if-env-changed=IPV4NUM_DB");

    let out_dir = env::var("OUT_DIR").unwrap();

    let database_str = match env::var("IPV4NUM_DB") {
        Ok(ip_csv_path) => {
            let attribution =
                env::var("IPV4NUM_DB_ATTRIBUTION").expect("IPV4NUM_DB_ATTRIBUTION must be set.");

            let db = Database::from_csv(ip_csv_path, Some(attribution)).expect("parse csv");

            #[cfg(windows)]
            let db_path = format!("{out_dir}/encoded_db")
                .replace(r"/", r"\")
                .replace(r"\", r"\\");
            #[cfg(not(windows))]
            let db_path = format!("{out_dir}/encoded_db");

            fs::write(
                &db_path,
                miniz_oxide::deflate::compress_to_vec(
                    &bincode::serialize(&db).expect("serialize db"),
                    10,
                ),
            )
            .expect("write db to file");

            format!(
                "Some(
                bincode::deserialize(
                    &miniz_oxide::inflate::decompress_to_vec(
                        include_bytes!(\"{db_path}\").as_slice()
                    ).expect(\"decompress database\")
                ).expect(\"deserialize database\")
            )"
            )
        }
        Err(_) => "None".to_string(),
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
