use std::{
    env,
    fs::File,
    io::{self, Write},
    os::windows::prelude::*,
    ptr,
};

use crate::{ChildError, Response};
use child_ipc::windows::pipe_name;
use windows_sys::Win32::{Foundation::*, Storage::FileSystem::*};

static mut PIPE: HANDLE = 0 as HANDLE;

pub fn init() {
    unsafe {
        let Some(pipe_name) = env::args()
            .nth(2)
            .and_then(|p| p.parse::<u64>().ok())
            .map(pipe_name)
        else {
            super::exit_with_error(Err(Error::Ipc("Invalid Pipe Name".into())))
        };
        let pipe_name_wide: Vec<u16> = child_ipc::wide_null(&pipe_name);

        let handle = CreateFileW(
            pipe_name_wide.as_ptr(),
            GENERIC_WRITE,
            0,
            ptr::null_mut(),
            OPEN_EXISTING,
            0,
            ptr::null_mut(),
        );

        if handle == INVALID_HANDLE_VALUE {
            panic!("Failed to open named pipe: {}", io::Error::last_os_error());
        }

        PIPE = handle;
    }
}

pub fn send_response(resp: Result<Response, ChildError>) {
    let mut file = unsafe { File::from_raw_handle(PIPE as _) };

    serde_json::to_writer(&file, &resp).unwrap();
    write!(file, "\r\n").unwrap();

    file.flush().unwrap();

    std::mem::forget(file);
}
