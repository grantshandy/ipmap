use std::{env, fs};

#[path = "src/database.rs"]
mod database;

use database::{CompactDatabase, IpType, Ipv4Bytes, Ipv6Bytes};

fn main() {
    tauri_build::build();

    let out_dir = env::var("OUT_DIR").unwrap();

    fs::write(
        format!("{out_dir}/internal_database.rs"),
        format!(
            r#"
                use std::sync::Arc;
                include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/database.rs"));
                lazy_static::lazy_static! {{
                    pub static ref IPV4_DATABASE: Option<Arc<Database<Ipv4Bytes>>> = {};
                    pub static ref IPV6_DATABASE: Option<Arc<Database<Ipv6Bytes>>> = {};
                }}
            "#,
            embed_database(&out_dir, IpType::IPv4),
            embed_database(&out_dir, IpType::IPv6)
        ),
    )
    .expect("open database file");
}

// Returns Some(...) or None
fn embed_database(out_dir: &String, kind: IpType) -> String {
    let db_var = match kind {
        IpType::IPv4 => "IPV4NUM_DB",
        IpType::IPv6 => "IPV6NUM_DB",
    };

    println!("cargo:rerun-if-env-changed={db_var}");

    match env::var(db_var) {
        Ok(ip_csv_path) => {
            let attribution = env::var(format!("{db_var}_ATTRIBUTION"))
                .expect("IPVX_DB_ATTRIBUTION must be set.");

            let db = match kind {
                IpType::IPv4 => bincode::serialize(
                    &CompactDatabase::<Ipv4Bytes>::from_csv(ip_csv_path, Some(attribution))
                        .expect("parse database"),
                ),
                IpType::IPv6 => bincode::serialize(
                    &CompactDatabase::<Ipv6Bytes>::from_csv(ip_csv_path, Some(attribution))
                        .expect("parse database"),
                ),
            }
            .expect("serialize db");

            #[cfg(windows)]
            let db_path = format!("{out_dir}/encoded_db_v4")
                .replace(r"/", r"\")
                .replace(r"\", r"\\");
            #[cfg(not(windows))]
            let db_path = format!("{out_dir}/encoded_db_v4");

            fs::write(
                &db_path,
                miniz_oxide::deflate::compress_to_vec(
                    &miniz_oxide::deflate::compress_to_vec(&db, 10),
                    10,
                ),
            )
            .expect("write db to file");

            let written_type = match kind {
                IpType::IPv4 => "Ipv4Bytes",
                IpType::IPv6 => "Ipv6Bytes",
            };

            format!(
                r#"Some(
                    Arc::new(
                        bincode::deserialize::<CompactDatabase<{written_type}>>(
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
                    )
                )"#
            )
        }
        Err(_) => "None".to_string(),
    }
}
