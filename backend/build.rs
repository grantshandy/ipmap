use std::{env, fs};

#[path = "src/database.rs"]
mod database;

use database::{CompactDatabase, Ipv4Bytes};

fn main() {
    tauri_build::build();

    println!("cargo:rerun-if-env-changed=IPV4NUM_DB");

    let out_dir = env::var("OUT_DIR").unwrap();

    let database_v4_str = env::var("IPV4NUM_DB")
        .map(|ip_csv_path| {
            let attribution =
                env::var("IPV4NUM_DB_ATTRIBUTION").expect("IPV4NUM_DB_ATTRIBUTION must be set.");

            let db = CompactDatabase::<Ipv4Bytes>::from_csv(ip_csv_path, Some(attribution))
                .expect("parse csv");

            #[cfg(windows)]
            let db_path = format!("{out_dir}/encoded_db_v4")
                .replace(r"/", r"\")
                .replace(r"\", r"\\");
            #[cfg(not(windows))]
            let db_path = format!("{out_dir}/encoded_db_v4");

            fs::write(
                &db_path,
                miniz_oxide::deflate::compress_to_vec(
                    &miniz_oxide::deflate::compress_to_vec(
                        &bincode::serialize(&db).expect("serialize db v4"),
                        10,
                    ),
                    10,
                ),
            )
            .expect("write db to file");

            format!(
                r#"Some(
                    bincode::deserialize::<CompactDatabase<Ipv4Bytes>>(
                        &miniz_oxide::inflate::decompress_to_vec(
                            &miniz_oxide::inflate::decompress_to_vec(
                                include_bytes!("{db_path}").as_slice()
                            )
                            .expect("decompress database 1")
                        )
                        .expect("decompress database 2")
                    )
                    .expect("deserialize database")
                    .into()
                )"#
            )
        })
        .unwrap_or("None".to_string());

    let database_v6_str = env::var("IPV6NUM_DB")
        .map(|ip_csv_path| {
            let attribution =
                env::var("IPV6NUM_DB_ATTRIBUTION").expect("IPV6NUM_DB_ATTRIBUTION must be set.");

            let db = CompactDatabase::<Ipv4Bytes>::from_csv(ip_csv_path, Some(attribution))
                .expect("parse csv");

            #[cfg(windows)]
            let db_path = format!("{out_dir}/encoded_db_v6")
                .replace(r"/", r"\")
                .replace(r"\", r"\\");
            #[cfg(not(windows))]
            let db_path = format!("{out_dir}/encoded_db_v6");

            fs::write(
                &db_path,
                miniz_oxide::deflate::compress_to_vec(
                    &miniz_oxide::deflate::compress_to_vec(
                        &bincode::serialize(&db).expect("serialize db v6"),
                        10,
                    ),
                    10,
                ),
            )
            .expect("write db to file");

            format!(
                r#"Some(
                    bincode::deserialize::<CompactDatabase<Ipv6Bytes>>(
                        &miniz_oxide::inflate::decompress_to_vec(
                            &miniz_oxide::inflate::decompress_to_vec(
                                include_bytes!("{db_path}").as_slice()
                            )
                            .expect("decompress database 1")
                        )
                        .expect("decompress database 2")
                    )
                    .expect("deserialize database")
                    .into()
                )"#
            )
        })
        .unwrap_or("None".to_string());

    fs::write(
        format!("{out_dir}/internal_database.rs"),
        format!(
            r#"
                use std::sync::Arc;
                include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/database.rs"));
                lazy_static::lazy_static! {{
                    pub static ref IPV4_DATABASE: Option<Arc<Database<Ipv4Bytes>>> = {database_v4_str};
                    pub static ref IPV6_DATABASE: Option<Arc<Database<Ipv6Bytes>>> = {database_v6_str};
                }}
            "#
        ),
    )
    .expect("open database file");
}
