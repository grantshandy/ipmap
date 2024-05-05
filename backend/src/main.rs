#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{AppHandle, Runtime, Window};

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![increment])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
async fn increment<R: Runtime>(app: AppHandle<R>, window: Window<R>) -> Result<(), String> {
  println!("increment!");

  Ok(())
}
