use std::{env, fs, path::PathBuf};

use child_ipc::EXE_NAME;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let from = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .join("..")
        .join("..")
        .join("target")
        .join("release")
        .join(EXE_NAME)
        .canonicalize()?;

    let to = PathBuf::new().join("resources").join(EXE_NAME);

    let _ = fs::create_dir("resources");

    fs::copy(from, to)?;

    tauri_build::build();

    Ok(())
}
