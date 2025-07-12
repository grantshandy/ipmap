use std::{env, process};

use base64::prelude::*;
use child_ipc::{ChildError, Command};

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

pub fn exit_with_error(error: ChildError) -> ! {
    send_response(Err(error));
    // nonzero would be correct, but harder to handle in the IPC layer.
    process::exit(0)
}

pub fn get_command() -> Command {
    let Some(message) = env::args().nth(1) else {
        exit_with_error(ChildError::Runtime(format!(
            "ipmap-child.exe must be provided with a command"
        )));
    };

    let decoded = BASE64_STANDARD.decode(&message).unwrap();

    match serde_json::from_slice::<Command>(&decoded) {
        Ok(cmd) => cmd,
        Err(err) => exit_with_error(ChildError::Runtime(format!(
            "Failed to parse command: {err}"
        ))),
    }
}
