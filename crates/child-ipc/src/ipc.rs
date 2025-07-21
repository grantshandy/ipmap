use crate::{Error, Response};

pub type Message = Result<Response, Error>;

#[cfg(feature = "child")]
pub use child::*;

#[cfg(feature = "child")]
mod child {
    use std::{env, process};

    use super::Message;
    use crate::{Command, Error, Response};
    use base64::prelude::*;

    pub type Parent = ipc_channel::ipc::IpcSender<Message>;

    pub fn get_command() -> (Command, Parent) {
        let mut args = env::args();

        let _exe_name = args.next();

        let Some(command) = args
            .next()
            .and_then(|n| BASE64_STANDARD.decode(n).ok())
            .and_then(|v| postcard::from_bytes(&v).ok())
        else {
            panic!("Invalid command argument");
        };

        let Some(channel_name) = args
            .next()
            .and_then(|n| BASE64_STANDARD.decode(n).ok())
            .and_then(|v| String::from_utf8(v).ok())
        else {
            panic!("Invalid channel argument");
        };

        let channel = match Parent::connect(channel_name) {
            Ok(channel) => channel,
            Err(error) => panic!("Couldn't connect to parent: {error}"),
        };

        // The parent requires we send this immediately after connecting.
        send_response(&channel, Ok(Response::Connected));

        (command, channel)
    }

    pub fn send_response(parent: &Parent, response: Message) {
        parent.send(response).unwrap();
    }

    pub fn exit_with_error(parent: &Parent, error: Error) -> ! {
        send_response(parent, Err(error));
        process::exit(1);
    }
}

#[cfg(feature = "parent")]
pub use parent::*;

#[cfg(feature = "parent")]
mod parent {
    use std::{io, path::PathBuf, process::Command as ProcessCommand, time::Duration};

    use base64::prelude::*;
    use ipc_channel::ipc::IpcOneShotServer;

    use super::Message;
    use crate::{Command, Error, ErrorKind, Response};

    const CHILD_CONNECTION_TIMEOUT: Duration = Duration::from_millis(200);

    pub type StopCallback = Box<dyn FnOnce() -> io::Result<()> + Send + Sync>;
    pub type Child = ipc_channel::ipc::IpcReceiver<Message>;

    pub async fn call_child_process(
        child_path: PathBuf,
        command: Command,
    ) -> Result<Response, Error> {
        let (recv, stop) = spawn_child_process(child_path, command).await?;

        let msg = recv.recv()?;

        stop()?;

        msg
    }

    pub async fn spawn_child_process(
        mut child_path: PathBuf,
        command: Command,
    ) -> Result<(Child, StopCallback), Error> {
        child_path = child_path.canonicalize()?;

        let (server, channel_name) = IpcOneShotServer::<Message>::new()?;

        let args = [
            BASE64_STANDARD.encode(postcard::to_allocvec(&command).unwrap()),
            BASE64_STANDARD.encode(channel_name),
        ];

        #[cfg(windows)]
        let stop = if command.needs_admin() {
            windows::spawn_child_admin(child_path, args)?
        } else {
            spawn_child(child_path, args)?
        };

        #[cfg(not(windows))]
        let stop = spawn_child(child_path, args)?;

        let accept = async { server.accept() };

        let (recv, first) = tokio::time::timeout(CHILD_CONNECTION_TIMEOUT, accept)
            .await
            .map_err(|_| Error::basic(ErrorKind::ChildTimeout))??;

        match first {
            Ok(Response::Connected) => Ok((recv, stop)),
            actual => {
                stop()?;
                Err(Error::message(
                    ErrorKind::EstablishConnection,
                    format!("Expected Ok(Response::Connected), got {actual:?}"),
                ))
            }
        }
    }

    fn spawn_child(child_path: PathBuf, args: [String; 2]) -> io::Result<StopCallback> {
        let mut child = ProcessCommand::new(child_path.clone()).args(args).spawn()?;

        Ok(Box::new(move || {
            match child.try_wait()? {
                Some(status) => {
                    if !status.success() {
                        return Err(io::Error::other(format!(
                            "Child process exited with code: {}",
                            status
                                .code()
                                .map_or("unknown".to_string(), |c| format!("{c:#x}"))
                        )));
                    }
                }
                None => {
                    // Still running, so terminate it
                    tracing::debug!("Terminating the child process...");
                    child.kill()?;
                    child.wait()?; // Ensure it fully exits
                }
            }

            tracing::debug!("{child_path:?} finished exiting");

            Ok(())
        }))
    }

    #[cfg(windows)]
    mod windows {
        use std::{ffi::OsStr, io, mem, os::windows::ffi::OsStrExt, path::PathBuf, ptr};

        use windows_sys::{
            Win32::{Foundation::*, System::Threading::*, UI::Shell::*},
            w,
        };

        use super::StopCallback;

        fn wide_null(s: impl AsRef<OsStr>) -> Vec<u16> {
            s.as_ref().encode_wide().chain(Some(0)).collect()
        }

        pub fn spawn_child_admin(
            child_path: PathBuf,
            args: [String; 2],
        ) -> io::Result<StopCallback> {
            let child_path = wide_null(child_path.clone());
            let args = wide_null(args.join(" "));

            let mut sei: SHELLEXECUTEINFOW = unsafe { mem::zeroed() };
            sei.cbSize = mem::size_of::<SHELLEXECUTEINFOW>() as u32;
            sei.hwnd = 0 as HWND; // No parent window
            sei.lpVerb = w!("runas");
            sei.lpClass = w!("exefile");
            sei.lpFile = child_path.as_ptr();
            sei.lpParameters = args.as_ptr();
            sei.lpDirectory = ptr::null(); // Current directory
            sei.nShow = 0; // SW_HIDE
            sei.fMask = SEE_MASK_NOCLOSEPROCESS | SEE_MASK_CLASSNAME;

            if unsafe { ShellExecuteExW(&mut sei) } == 0 {
                return Err(io::Error::last_os_error());
            }

            let process_handle: HANDLE = sei.hProcess;

            if process_handle.is_null() {
                return Err(io::Error::other("Process was not started."));
            }

            // SAFETY: HANDLE is a pointer, but we can cast it to usize for Send + Sync closure.
            let process_handle_usize = process_handle as usize;

            Ok(Box::new(move || {
                let process_handle = process_handle_usize as HANDLE;

                // Get the exit code to see the status of the program
                let mut exit_code = 0;
                if unsafe { GetExitCodeProcess(process_handle, &mut exit_code) } == 0 {
                    unsafe { CloseHandle(process_handle) };
                    return Err(io::Error::last_os_error());
                }

                let mut was_terminated = false;

                // If the program is still running, terminate it.
                if exit_code == STILL_ACTIVE as u32 {
                    tracing::debug!("terminating the child process...");

                    if unsafe { TerminateProcess(process_handle, 1) } == 0 {
                        let err = io::Error::last_os_error();

                        unsafe { CloseHandle(process_handle) };

                        return Err(io::Error::other(format!(
                            "Failed to terminate process: {err}"
                        )));
                    }

                    was_terminated = true;
                }

                // Wait for the process to fully exit.
                unsafe { WaitForSingleObject(process_handle, INFINITE) };

                // Get final exit code after termination or natural exit.
                if unsafe { GetExitCodeProcess(process_handle, &mut exit_code) } == 0 {
                    unsafe { CloseHandle(process_handle) };
                    return Err(io::Error::last_os_error());
                }

                if unsafe { CloseHandle(process_handle) } == 0 {
                    return Err(io::Error::last_os_error());
                }

                tracing::debug!("{child_path:?} finished exiting with code: {exit_code:#x}");

                // if we terminated the child the exit code will be 0x103 which is expected,
                // If the process naturally failed on its own, return that as an error.
                if !was_terminated && exit_code != 0 {
                    Err(io::Error::other(format!(
                        "Child process exited with code: {exit_code:#x}"
                    )))
                } else {
                    Ok(())
                }
            }))
        }
    }
}
