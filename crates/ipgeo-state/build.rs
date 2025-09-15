use std::{env, fs, io::Write, path::PathBuf};

use flate2::{Compression, write::GzEncoder};
use ipgeo::GenericDatabase;

#[path = "src/preloads/shared.rs"]
mod shared;

const IN_ENV: &str = "DB_PRELOADS";

#[cfg(unix)]
const SEPARATOR: &str = ":";

#[cfg(windows)]
const SEPARATOR: &str = ";";

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed={IN_ENV}");
    println!("cargo::rustc-check-cfg=cfg(db_preloads)");

    let Ok(db_preloads) = env::var(IN_ENV) else {
        return Ok(());
    };

    if db_preloads.is_empty() {
        return Ok(());
    }

    let mut ipv4 = Vec::new();
    let mut ipv6 = Vec::new();

    for db in db_preloads.split(SEPARATOR).map(parse_db_preload_path) {
        let (path, db) = db?;

        let path = path
            .file_name()
            .expect("DB_PRELOADS must contain files not paths")
            .into();

        match db {
            GenericDatabase::Ipv4(db) => ipv4.push((path, db)),
            GenericDatabase::Ipv6(db) => ipv6.push((path, db)),
        }
    }

    let encoded = postcard::to_allocvec::<shared::DiskDatabases>(&(ipv4, ipv6))?;

    let mut compressor = GzEncoder::new(Vec::new(), Compression::best());
    compressor.write_all(&encoded)?;

    let dest_path = PathBuf::from(env::var("OUT_DIR")?).join("db_preloads.bin");
    fs::write(&dest_path, compressor.finish()?)?;

    println!("cargo:rustc-cfg=db_preloads");
    println!(
        "cargo:rustc-env=DB_PRELOADS_BIN={}",
        dest_path.to_string_lossy()
    );

    Ok(())
}

fn parse_db_preload_path(path: &str) -> Result<(PathBuf, GenericDatabase)> {
    let path = PathBuf::from(path).canonicalize()?;
    println!("cargo:rerun-if-changed={}", path.to_string_lossy());

    let db = ipgeo::detect(&path)?;

    Ok((path, db))
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
