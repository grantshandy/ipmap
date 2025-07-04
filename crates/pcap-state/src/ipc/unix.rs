use std::{
    io::{self, BufRead, BufReader},
    process::{Command as ProcessCommand, Stdio},
};

use child_ipc::Command;

use crate::{StopCallback, ipc};

pub const EXE_NAME: &str = "ipmap-child";

pub fn spawn_child_process(
    command: Command,
    _admin: bool,
) -> io::Result<(impl BufRead, StopCallback)> {
    let child_path = ipc::find_isolate_child()?;

    let mut child = ProcessCommand::new(child_path)
        .arg(super::command_to_string(command))
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
