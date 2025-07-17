use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

const ABOUT_WINDOW_ID: &str = "about";
const ABOUT_WINDOW_OFFSET: i32 = 50;

#[specta::specta]
#[tauri::command]
pub async fn open_about_window(app: AppHandle) {
    if app
        .webview_windows()
        .keys()
        .any(|label| label.as_str() == ABOUT_WINDOW_ID)
    {
        return;
    }

    let mut w = WebviewWindowBuilder::new(&app, ABOUT_WINDOW_ID, WebviewUrl::App("about".into()))
        .title("About")
        .minimizable(false)
        .maximizable(false)
        .inner_size(350.0, 400.0)
        .resizable(false);

    if let Some(main) = app.get_webview_window("main") {
        w = w.parent(&main).unwrap();

        let pos = main.outer_position().unwrap();

        w = w.position(
            (pos.x + ABOUT_WINDOW_OFFSET) as f64,
            (pos.y + ABOUT_WINDOW_OFFSET) as f64,
        );
    }

    w.build().unwrap();
}

#[tauri::command]
#[specta::specta]
pub fn platform() -> Platform {
    #[cfg(target_os = "linux")]
    return Platform::Linux;

    #[cfg(target_os = "windows")]
    return Platform::Windows;

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    return Platform::MacOS;
}

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Linux,
    Windows,
    MacOS,
}

#[tauri::command]
#[specta::specta]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
