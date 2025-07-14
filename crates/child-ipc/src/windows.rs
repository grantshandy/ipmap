use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

pub fn wide_null(s: impl AsRef<OsStr>) -> Vec<u16> {
    s.as_ref().encode_wide().chain(Some(0)).collect()
}

pub fn pipe_name(id: u64) -> String {
    format!(r"\\.\pipe\ipmap-{id}")
}
