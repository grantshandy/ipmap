use std::{env, error, fs};

use sha2::{Digest, Sha256};

#[path = "src/child.rs"]
mod child;

const COMMANDS: &[&str] = &[
    "start_capture",
    "stop_capture",
    "init_pcap",
    "traceroute_enabled",
    "run_traceroute",
    "print_error",
    "my_location",
];

fn main() -> Result<(), Box<dyn error::Error>> {
    tauri_plugin::Builder::new(COMMANDS).build();

    fs::write(format!("{}/ipmap-child.sha256", env::var("OUT_DIR")?), {
        let mut hasher = Sha256::new();
        hasher.update(child::CHILD_BYTES);
        hasher.finalize()
    })?;

    Ok(())
}
