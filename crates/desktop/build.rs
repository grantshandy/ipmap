use std::{env, fs, path::PathBuf};

use child_ipc::EXE_NAME;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let platform_exe = if cfg!(target_os = "windows") {
        format!("{EXE_NAME}.exe")
    } else {
        EXE_NAME.to_string()
    };

    let from = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .parent().unwrap()
        .parent().unwrap()
        .join("target")
        .join("release")
        .join(platform_exe)
        .canonicalize()?;

    let to = PathBuf::new().join("resources").join(EXE_NAME);

    let _ = fs::create_dir("resources");

    fs::copy(from, to)?;

    tauri_build::build();

    Ok(())
}
