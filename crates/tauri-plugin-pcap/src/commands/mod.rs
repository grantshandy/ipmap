use child_ipc::Error;

mod capture;
mod my_location;
mod traceroute;

pub use capture::*;
pub use my_location::*;
pub use traceroute::*;

#[tauri::command]
#[specta::specta]
pub fn print_error(error: Error) -> String {
    error.to_string()
}
