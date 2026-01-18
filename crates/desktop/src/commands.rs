use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

const ABOUT_WINDOW_ID: &str = "about";
const ABOUT_WINDOW_OFFSET: i32 = 50;

#[specta::specta]
#[tauri::command]
pub async fn open_about_window<R: Runtime>(app: AppHandle<R>) {
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
        .inner_size(350.0, 500.0)
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

#[allow(dead_code)]
#[derive(serde::Serialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Linux,
    Windows,
    MacOS,
}

#[allow(unreachable_code)]
#[specta::specta]
#[tauri::command]
pub fn platform() -> Platform {
    #[cfg(target_os = "linux")]
    return Platform::Linux;

    #[cfg(target_os = "windows")]
    return Platform::Windows;

    #[cfg(target_os = "macos")]
    return Platform::MacOS;

    unimplemented!("Only Linux, Windows, and MacOS are currently supported")
}
