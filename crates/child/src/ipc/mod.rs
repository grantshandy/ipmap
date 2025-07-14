use std::{env, process};

use child_ipc::{ChildError, Command};

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::send_response;

#[cfg(windows)]
pub mod windows;
#[cfg(windows)]
pub use windows::send_response;

pub fn exit_with_error(error: ChildError) -> ! {
    send_response(Err(error));
    // nonzero would be correct, but harder to handle in the IPC layer.
    process::exit(0)
}

pub fn get_command() -> Command {
    let Some(message) = env::args().nth(1) else {
        exit_with_error(ChildError::Runtime(format!(
            "ipmap-child must be provided with a command"
        )));
    };

    match Command::from_arg_string(message) {
        Some(c) => c,
        None => exit_with_error(ChildError::Runtime("Invalid first argument".into())),
    }
}
