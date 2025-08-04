use std::{fmt, io, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum ErrorKind {
    UnexpectedType,
    TerminatedUnexpectedly,
    ChildTimeout,
    Ipc,
    InsufficientPermissions,
    LibLoading,
    Runtime,
    ChildNotFound,
    EstablishConnection,
    Io,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            ErrorKind::UnexpectedType => "Unexpected type returned from child",
            ErrorKind::TerminatedUnexpectedly => "The child process terminated unexpectedly",
            ErrorKind::Ipc => "IPC error",
            ErrorKind::InsufficientPermissions => "Insufficient network permissions",
            ErrorKind::LibLoading => "Error loading libpcap",
            ErrorKind::Runtime => "Runtime error",
            ErrorKind::ChildNotFound => "Child process not found",
            ErrorKind::EstablishConnection => "Failed to establish connection",
            ErrorKind::Io => "IO error",
            ErrorKind::ChildTimeout => "The child failed to connect to the IPC pipe in time",
        };

        f.write_str(msg)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Error {
    pub kind: ErrorKind,
    pub message: Option<String>,
}

impl Error {
    pub fn basic(kind: ErrorKind) -> Self {
        Self {
            kind,
            message: None,
        }
    }

    pub fn message(kind: ErrorKind, message: String) -> Self {
        Self {
            kind,
            message: Some(message),
        }
    }

    pub fn insufficient_permissions(path: PathBuf) -> Self {
        Self {
            kind: ErrorKind::InsufficientPermissions,
            message: Some(path.display().to_string()),
        }
    }

    pub fn runtime(msg: String) -> Self {
        Self {
            kind: ErrorKind::Runtime,
            message: Some(msg),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self {
            kind: ErrorKind::Io,
            message: Some(value.to_string()),
        }
    }
}

#[cfg(feature = "parent")]
impl From<ipc_channel::ipc::IpcError> for Error {
    fn from(value: ipc_channel::ipc::IpcError) -> Self {
        Self {
            kind: ErrorKind::Ipc,
            message: Some(value.to_string()),
        }
    }
}

#[cfg(feature = "parent")]
impl From<ipc_channel::Error> for Error {
    fn from(value: ipc_channel::Error) -> Self {
        Self {
            kind: ErrorKind::Ipc,
            message: Some(value.to_string()),
        }
    }
}

// Implement Display for the Error struct using its message field
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(msg) => write!(f, "{}: {}", self.kind, msg),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl std::error::Error for Error {}
