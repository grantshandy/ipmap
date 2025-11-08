use child_ipc::Error;

pub mod capture;
pub mod traceroute;

pub use capture::*;
pub use traceroute::*;

#[tauri::command]
#[specta::specta]
pub fn print_error(error: Error) -> String {
    error.to_string()
}
