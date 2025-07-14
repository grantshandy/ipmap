use std::{
    io::{self, BufRead, BufReader},
    path::PathBuf,
    process::{Command as ProcessCommand, Stdio},
};

use child_ipc::{Command, EXE_NAME};

use crate::StopCallback;

pub fn spawn_child_process(
    child_path: PathBuf,
    command: Command,
    _admin: bool,
) -> io::Result<(impl BufRead, StopCallback)> {
    tracing::debug!("Calling {child_path:?} with {command:?}");

    let mut child = ProcessCommand::new(child_path)
        .arg(command.to_arg_string())
        .stdout(Stdio::piped())
        .spawn()?;

    let Some(stdout) = child.stdout.take() else {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to get stdout"));
    };

    let exit_signal = Box::new(move || {
        child.kill()?;
        child.wait()?;
        tracing::debug!("{EXE_NAME} finished exiting");
        Ok(())
    });

    Ok((BufReader::new(stdout), exit_signal))
}
