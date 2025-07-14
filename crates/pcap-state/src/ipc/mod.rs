use std::{
    io::{self, BufRead, Read},
    path::PathBuf,
    rc::Rc,
};

use crate::StopCallback;
use child_ipc::{ChildError, Command, EXE_NAME, Response};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{
    Runtime,
    path::{BaseDirectory, PathResolver},
};

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows::*;

pub fn call_child_process(
    child_path: PathBuf,
    command: Command,
    admin: bool,
) -> Result<Response, Error> {
    let (mut reader, exit) = spawn_child_process(child_path.clone(), command, admin)
        .map_err(|e| Error::Ipc(e.to_string()))?;

    let mut output: Vec<u8> = Vec::new();
    reader
        .read_to_end(&mut output)
        .map_err(|e| Error::Ipc(e.to_string()))?;

    exit().map_err(|e| Error::Ipc(e.to_string()))?;

    serde_json::from_slice::<Result<Response, ChildError>>(&output)
        .map_err(|e| Error::Ipc(e.to_string()))?
        .map_err(|e| map_child_error(e, &child_path))
}

pub fn spawn_child_iterator(
    child_path: PathBuf,
    command: Command,
    admin: bool,
) -> io::Result<(impl Iterator<Item = Result<Response, Error>>, StopCallback)> {
    let (reader, exit_signal) = spawn_child_process(child_path.clone(), command, admin)?;

    let path = Rc::new(child_path);

    // Process should only emit Result<Response, Error> as JSON strings separated by newlines.
    let iter = reader
        .lines()
        .map(|line| line.map_err(|e| Error::Ipc(e.to_string())))
        .map(move |line| {
            line.and_then(
                |l| match serde_json::from_str::<Result<Response, ChildError>>(&l) {
                    Ok(resp) => resp.map_err(|e| map_child_error(e, &path.clone())),
                    Err(err) => Err(Error::Ipc(err.to_string())),
                },
            )
        });

    Ok((iter, exit_signal))
}

pub(crate) fn resolve_child_path(resolver: &PathResolver<impl Runtime>) -> Result<PathBuf, String> {
    resolver
        .resolve(
            PathBuf::from("resources").join(EXE_NAME),
            BaseDirectory::Resource,
        )
        .map_err(|e| format!("{EXE_NAME} not found: {e}"))
}

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error, Type)]
#[serde(tag = "t", content = "c")]
pub enum Error {
    #[error("Insufficient network permissions on pcap-child process")]
    InsufficientPermissions(PathBuf),
    #[error("Libpcap loading error: {0}")]
    LibLoading(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("IPC error: {0}")]
    Ipc(String),
    #[error("Child Executable not found: {0}")]
    NotFound(String),
}

pub fn map_child_error(err: ChildError, child: &PathBuf) -> Error {
    match err {
        ChildError::InsufficientPermissions => Error::InsufficientPermissions(child.clone()),
        ChildError::LibLoading(e) => Error::LibLoading(e),
        ChildError::Runtime(e) => Error::Runtime(e),
    }
}
