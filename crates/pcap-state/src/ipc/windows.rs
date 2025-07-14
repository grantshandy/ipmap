use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    os::windows::io::FromRawHandle,
    process::Command as ProcessCommand,
    path::PathBuf,
    ptr,
};

use crate::{StopCallback, ipc};
use child_ipc::{
    Command, EXE_NAME,
    windows::{pipe_name, wide_null},
};

use windows_sys::Win32::{
    Foundation::*,
    Storage::FileSystem::*,
    System::{Pipes::*, Threading::*},
    UI::Shell::*,
};

pub fn spawn_child_process(
    child_path: PathBuf,
    command: Command,
    admin: bool,
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

    let exit_signal = if admin {
        spawn_admin_process(child_path, pipe_id, command)?
    } else {
        spawn_normal_process(child_path, pipe_id, command)?
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
    pipe_id: u64,
    command: Command,
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
    pipe_id: u64,
    command: Command,
) -> io::Result<StopCallback> {
    let exe_wide = wide_null(child_path);
    let verb = wide_null("runas");
    let params = wide_null(format!("{} {pipe_id}", command.to_arg_string()));

    let mut sei: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
    sei.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    sei.fMask = SEE_MASK_NOCLOSEPROCESS; // Important: Get the process handle
    sei.hwnd = 0 as HWND; // No parent window
    sei.lpVerb = verb.as_ptr();
    sei.lpFile = exe_wide.as_ptr();
    sei.lpParameters = params.as_ptr();
    sei.lpDirectory = std::ptr::null(); // Current directory
    sei.nShow = 1; // SW_SHOWNORMAL (show the window normally)

    let success = unsafe { ShellExecuteExW(&mut sei) };

    if success == 0 {
        return Err(io::Error::from_raw_os_error(
            unsafe { GetLastError() } as i32
        ));
    }

    let process_handle: HANDLE = sei.hProcess;

    if process_handle.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Process was not started.",
        ));
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
