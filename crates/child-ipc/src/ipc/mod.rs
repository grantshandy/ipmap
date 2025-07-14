use std::io;

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

pub type StopCallback = Box<dyn FnOnce() -> io::Result<()> + Send + Sync>;

#[cfg(feature = "parent")]
pub use parent::*;

#[cfg(feature = "parent")]
mod parent {
    #[cfg(unix)]
    pub use super::unix::parent::*;
    #[cfg(windows)]
    pub use super::windows::parent::*;

    use super::StopCallback;
    use crate::{ChildError, Command, EXE_NAME, Error, Response};
    use std::{
        io::{self, BufRead},
        path::PathBuf,
        rc::Rc,
    };

    pub fn call_child_process(child_path: PathBuf, command: Command) -> Result<Response, Error> {
        let (mut iter, stop) =
            spawn_child_iterator(child_path, command).map_err(|e| Error::Ipc(e.to_string()))?;

        let res = iter
            .next()
            .ok_or(Error::Ipc(format!("No response from {EXE_NAME}")))?;

        stop().map_err(|e| Error::Ipc(format!("Failed to stop {EXE_NAME}: {e}")))?;

        res
    }

    pub fn spawn_child_iterator(
        child_path: PathBuf,
        command: Command,
    ) -> io::Result<(impl Iterator<Item = Result<Response, Error>>, StopCallback)> {
        tracing::debug!("Calling {child_path:?} with {command:?}");

        let (reader, exit_signal) = spawn_child_process(child_path.clone(), command)?;

        let path = Rc::new(child_path);

        // Process should only emit Result<Response, Error> as JSON strings separated by newlines.
        let iter = reader
            .lines()
            .map(|line| line.map_err(|e| Error::Ipc(e.to_string())))
            .map(move |line| {
                line.and_then(
                    |l| match serde_json::from_str::<Result<Response, ChildError>>(&l) {
                        Ok(resp) => resp.map_err(|e| e.to_error(&path.clone())),
                        Err(err) => Err(Error::Ipc(err.to_string())),
                    },
                )
            });

        Ok((iter, exit_signal))
    }
}

#[cfg(feature = "child")]
pub use child::*;

#[cfg(feature = "child")]
mod child {
    use std::{env, process};

    use crate::{ChildError, Command, EXE_NAME};

    #[cfg(unix)]
    pub use super::unix::child::*;
    #[cfg(windows)]
    pub use super::windows::child::*;

    pub fn get_command() -> Command {
        let Some(message) = env::args().nth(1) else {
            exit_with_error(ChildError::Runtime(format!(
                "{EXE_NAME} must be provided with a command"
            )));
        };

        match Command::from_arg_string(message) {
            Some(c) => c,
            None => exit_with_error(ChildError::Runtime("Invalid first argument".into())),
        }
    }

    pub fn exit_with_error(error: ChildError) -> ! {
        send_response(Err(error));
        // nonzero would be correct, but harder to handle in the IPC layer.
        process::exit(0)
    }
}
