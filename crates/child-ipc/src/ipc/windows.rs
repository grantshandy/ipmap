use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

fn wide_null(s: impl AsRef<OsStr>) -> Vec<u16> {
    s.as_ref().encode_wide().chain(Some(0)).collect()
}

fn pipe_name(id: u64) -> String {
    format!(r"\\.\pipe\ipmap-{id}")
}

#[cfg(feature = "child")]
pub mod child {
    use std::{
        env,
        fs::File,
        io::{self, Write},
        os::windows::io::FromRawHandle,
        ptr,
    };

    use super::{pipe_name, wide_null};
    use crate::{ChildError, Response};
    use windows_sys::Win32::{Foundation::*, Storage::FileSystem::*};

    static mut PIPE: HANDLE = 0 as HANDLE;

    unsafe fn init_pipe() {
        let Some(pipe_name) = env::args()
            .nth(2)
            .and_then(|p| p.parse::<u64>().ok())
            .map(pipe_name)
        else {
            crate::ipc::exit_with_error(ChildError::Runtime("Invalid Pipe Name".into()))
        };

        let pipe_name_wide: Vec<u16> = wide_null(&pipe_name);

        let handle = unsafe {
            CreateFileW(
                pipe_name_wide.as_ptr(),
                GENERIC_WRITE,
                0,
                ptr::null_mut(),
                OPEN_EXISTING,
                0,
                ptr::null_mut(),
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            panic!("Failed to open named pipe: {}", io::Error::last_os_error());
        }

        unsafe {
            PIPE = handle;
        }
    }

    pub fn send_response(resp: Result<Response, ChildError>) {
        unsafe {
            if PIPE == 0 as HANDLE {
                init_pipe();
            }
        }

        let mut file = unsafe { File::from_raw_handle(PIPE as _) };

        serde_json::to_writer(&file, &resp).unwrap();
        write!(file, "\r\n").unwrap();
        file.flush().unwrap();

        std::mem::forget(file);
    }
}

#[cfg(feature = "parent")]
pub mod parent {
    use std::{
        fs::File,
        io::{self, BufRead, BufReader},
        os::windows::io::FromRawHandle,
        path::PathBuf,
        process::Command as ProcessCommand,
        ptr,
    };

    use super::{pipe_name, wide_null};
    use crate::{Command, EXE_NAME, ipc::StopCallback};

    use windows_sys::{
        Win32::{
            Foundation::*,
            Storage::FileSystem::*,
            System::{Pipes::*, Threading::*},
            UI::Shell::*,
        },
        w,
    };

    pub fn spawn_child_process(
        child_path: PathBuf,
        command: Command,
    ) -> io::Result<(impl BufRead, StopCallback)> {
        let pipe_id = fastrand::u64(..);
        let pipe_name = pipe_name(pipe_id);
        let pipe_name_wide = wide_null(&pipe_name);

        let pipe_handle = unsafe {
            CreateNamedPipeW(
                pipe_name_wide.as_ptr(),
                PIPE_ACCESS_INBOUND, // Only allow reading from the client
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                1,
                0,        // Out buffer (not used)
                u32::MAX, // In buffer (for reading from client)
                0,
                ptr::null(),
            )
        };

        if pipe_handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }

        let exit_signal = if command.needs_admin() {
            spawn_admin_process(child_path, command, pipe_id)?
        } else {
            spawn_normal_process(child_path, command, pipe_id)?
        };

        let connected = unsafe { ConnectNamedPipe(pipe_handle, ptr::null_mut()) };
        if connected == 0 {
            let err = unsafe { GetLastError() };
            if err != ERROR_PIPE_CONNECTED {
                return Err(io::Error::from_raw_os_error(err as i32));
            }
        }

        let file = unsafe { File::from_raw_handle(pipe_handle as *mut _) };
        let reader = BufReader::new(file);

        Ok((reader, exit_signal))
    }

    fn spawn_normal_process(
        child_path: PathBuf,
        command: Command,
        pipe_id: u64,
    ) -> io::Result<StopCallback> {
        let mut child = ProcessCommand::new(child_path)
            .arg(command.to_arg_string())
            .arg(pipe_id.to_string())
            .spawn()?;

        Ok(Box::new(move || {
            child.kill()?;
            child.wait()?;
            tracing::debug!("{EXE_NAME} finished exiting");
            Ok(())
        }))
    }

    fn spawn_admin_process(
        child_path: PathBuf,
        command: Command,
        pipe_id: u64,
    ) -> io::Result<StopCallback> {
        let exe_wide = wide_null(child_path);
        let params = wide_null(format!("{} {pipe_id}", command.to_arg_string()));

        let mut sei: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
        sei.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
        sei.hwnd = 0 as HWND; // No parent window
        sei.lpVerb = w!("runas");
        sei.lpFile = exe_wide.as_ptr();
        sei.lpParameters = params.as_ptr();
        sei.lpDirectory = ptr::null(); // Current directory
        sei.nShow = 0; // SW_HIDE
        sei.lpClass = w!("exefile");
        sei.fMask = SEE_MASK_NOCLOSEPROCESS | SEE_MASK_CLASSNAME;

        let success = unsafe { ShellExecuteExW(&mut sei) };

        if success == 0 {
            return Err(io::Error::from_raw_os_error(
                unsafe { GetLastError() } as i32
            ));
        }

        let process_handle: HANDLE = sei.hProcess;

        if process_handle.is_null() {
            return Err(io::Error::other("Process was not started."));
        }

        // SAFETY: HANDLE is a pointer, but we can cast it to usize for Send + Sync closure.
        let process_handle_usize = process_handle as usize;

        Ok(Box::new(move || {
            let process_handle = process_handle_usize as HANDLE;
            if unsafe { TerminateProcess(process_handle, 0) } == 0 {
                let code = unsafe { GetLastError() } as i32;
                if code != 5 {
                    return Err(io::Error::from_raw_os_error(code));
                } else {
                    return Ok(());
                }
            }

            unsafe { WaitForSingleObject(process_handle, INFINITE) };
            unsafe { CloseHandle(process_handle) };

            tracing::debug!("{EXE_NAME} finished exiting");
            Ok(())
        }))
    }
}
