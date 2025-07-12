use std::{fs, path::PathBuf, process::Command};

use child_ipc::EXE_NAME;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()?;

    let args = "cargo b -r -p ipmap-child";

    #[cfg(windows)]
    Command::new("cmd")
        .current_dir(&workspace_root)
        .args(["/C", args])
        .output()?;
    #[cfg(not(windows))]
    Command::new("sh")
        .current_dir(&workspace_root)
        .args(["-c", args])
        .output()?;

    let _ = fs::create_dir("resources");

    fs::copy(
        workspace_root.join("target").join("release").join(EXE_NAME),
        PathBuf::new().join("resources").join(EXE_NAME),
    )?;

    tauri_build::build();

    Ok(())
}
