// use std::{
//     env,
//     fs::{self, File},
//     io::Read,
//     path::PathBuf,
// };

// use ipgeo::GenericDatabase;

// const IN_ENV: &str = "DB_PRELOADS";

type StdError = Box<dyn std::error::Error>;

fn main() -> Result<(), StdError> {
    // println!("cargo:rerun-if-changed=build.rs");
    // println!("cargo:rerun-if-env-changed={IN_ENV}");

    // let Ok(db_preloads) = env::var(IN_ENV) else {
    //     return Ok(());
    // };

    // let dbs: Vec<GenericDatabase> = db_preloads
    //     .split(":")
    //     .map(parse_db_preload_path)
    //     .collect::<Result<_, StdError>>()?;

    // let encoded = postcard::to_allocvec(&dbs)?;

    // let dest_path = PathBuf::from(env::var("OUT_DIR")?).join("db_preloads.bin");
    // fs::write(&dest_path, encoded)?;

    // println!("cargo:rustc-cfg=db_preloads");
    // println!("cargo:rustc-env=DB_PRELOADS_BIN={dest_path:?}");

    Ok(())
}

// fn parse_db_preload_path(path: &str) -> Result<GenericDatabase, StdError> {
//     let path = PathBuf::from(path).canonicalize()?;
//     println!("cargo:rerun-if-changed={path:?}");

//     let db = ipgeo::from_read(File::open(path)?)?;

//     Ok(db)
// }
