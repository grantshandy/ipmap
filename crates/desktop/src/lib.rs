use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

const ABOUT_WINDOW_ID: &str = "about";
const ABOUT_WINDOW_OFFSET: i32 = 50;

#[specta::specta]
#[tauri::command]
async fn open_about_window<R: Runtime>(app: AppHandle<R>) {
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

fn builder() -> tauri_specta::Builder {
    tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
        open_about_window::<tauri::Wry>
    ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    let builder = builder();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_ipgeo::init())
        .plugin(tauri_plugin_pcap::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn export_types() {
        builder()
            .typ::<Platform>()
            .constant("PLATFORM", Platform::current())
            .constant("APP_VERSION", env!("CARGO_PKG_VERSION"))
            .error_handling(tauri_specta::ErrorHandlingMode::Result)
            .export(
                specta_typescript::Typescript::default()
                    .bigint(specta_typescript::BigIntExportBehavior::Number),
                "bindings/raw.ts",
            )
            .unwrap();
    }

    #[allow(dead_code)]
    #[derive(serde::Serialize, specta::Type)]
    #[serde(rename_all = "lowercase")]
    pub enum Platform {
        Linux,
        Windows,
        MacOS,
    }

    impl Platform {
        #[allow(unreachable_code)]
        pub const fn current() -> Self {
            #[cfg(target_os = "linux")]
            return Platform::Linux;

            #[cfg(target_os = "windows")]
            return Platform::Windows;

            #[cfg(target_os = "macos")]
            return Platform::MacOS;

            unimplemented!("Only Linux, Windows, and MacOS are currently supported")
        }
    }
}
