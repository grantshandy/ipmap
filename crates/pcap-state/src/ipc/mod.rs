use std::{
    env,
    io::{self, BufRead, Read},
    path::{Path, PathBuf},
};

use crate::StopCallback;
use base64::prelude::*;
use child_ipc::{Command, Error, Response};

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows::*;

pub fn call_child_process(command: Command, admin: bool) -> Result<Response, Error> {
    let (mut reader, exit) =
        spawn_child_process(command, admin).map_err(|e| Error::Ipc(e.to_string()))?;

    let mut output: Vec<u8> = Vec::new();
    reader
        .read_to_end(&mut output)
        .map_err(|e| Error::Ipc(e.to_string()))?;

    exit().map_err(|e| Error::Ipc(e.to_string()))?;

    serde_json::from_slice(&output).map_err(|e| Error::Ipc(e.to_string()))?
}

pub fn spawn_child_iterator(
    command: Command,
    admin: bool,
) -> io::Result<(impl Iterator<Item = Result<Response, Error>>, StopCallback)> {
    let (reader, exit_signal) = spawn_child_process(command, admin)?;

    // Process should only emit Result<Response, Error> as JSON strings separated by newlines.
    let iter = reader
        .lines()
        .map(|line| line.map_err(|e| Error::Ipc(e.to_string())))
        .map(|line| {
            line.and_then(
                |l| match serde_json::from_str::<Result<Response, Error>>(&l) {
                    Ok(resp) => resp,
                    Err(err) => Err(Error::Ipc(err.to_string())),
                },
            )
        });

    Ok((iter, exit_signal))
}

pub(crate) fn find_isolate_child() -> io::Result<PathBuf> {
    fn find() -> Option<PathBuf> {
        if let Ok(env) = env::var("IPMAP_CHILD") {
            let candidate = Path::new(&env);

            if candidate.exists() {
                return Some(candidate.to_path_buf());
            } else {
                tracing::warn!("{EXE_NAME} '{candidate:?}' doesn't exist, not using.");
            }
        }

        // 1. Next to current executable
        if let Ok(current_exe) = env::current_exe() {
            if let Some(dir) = current_exe.parent() {
                let candidate = dir.join(EXE_NAME);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }

        // 2. In PATH
        if let Ok(paths) = env::var("PATH") {
            for path in env::split_paths(&paths) {
                let candidate = path.join(EXE_NAME);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }

        // 3. In target/debug/ or target/release/ (for development)
        if let Ok(current) = env::current_dir() {
            let target = current.join("target");

            let debug = target.join("debug").join(EXE_NAME);
            if debug.exists() {
                return Some(debug);
            }

            let release = target.join("release").join(EXE_NAME);
            if release.exists() {
                return Some(release);
            }
        }

        None
    }

    find().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("{EXE_NAME} not found")))
}

pub(crate) fn command_to_string(command: Command) -> String {
    BASE64_STANDARD.encode(serde_json::to_string(&command).expect("Failed to serialize command"))
}
