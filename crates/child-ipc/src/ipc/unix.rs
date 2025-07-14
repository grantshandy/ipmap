#[cfg(feature = "parent")]
pub mod parent {
    use std::{
        io::{self, BufRead, BufReader},
        path::PathBuf,
        process::{Command as ProcessCommand, Stdio},
    };

    use crate::ipc::StopCallback;
    use crate::{Command, EXE_NAME};

    pub fn spawn_child_process(
        child_path: PathBuf,
        command: Command,
    ) -> io::Result<(impl BufRead, StopCallback)> {
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
}

#[cfg(feature = "child")]
pub mod child {
    use crate::{ChildError, Response};

    pub fn send_response(resp: Result<Response, ChildError>) {
        let s = serde_json::to_string(&resp).unwrap();
        println!("{s}");
    }
}
