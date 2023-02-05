#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{env, fs, net::IpAddr, path::PathBuf, sync::Arc, thread};

use etherparse::{InternetSlice, SlicedPacket};
use microkv::{errors as kverror, MicroKV};
use pcap::{Active, Capture, Device};
use tauri::{
    api::dialog::{blocking::MessageDialogBuilder, MessageDialogButtons, MessageDialogKind},
    AppHandle, Builder, CustomMenuItem, Manager, Menu, MenuItem, Submenu, WindowBuilder, WindowUrl,
};

fn main() {
    Builder::default()
        .menu(
            Menu::new()
                .add_submenu(Submenu::new(
                    "File",
                    Menu::new().add_native_item(MenuItem::Quit),
                ))
                .add_submenu(Submenu::new(
                    "Help",
                    Menu::new().add_item(CustomMenuItem::new("about", "About")),
                )),
        )
        .on_menu_event(|event| match event.menu_item_id() {
            "about" if event.window().get_window("about").is_none() => {
                thread::spawn(move || about_window(event.window().app_handle()));
            }
            _ => (),
        })
        .setup(|app| {
            // get database handle
            let db: MicroKV = match get_db() {
                Ok(db) => db,
                Err(err) => {
                    MessageDialogBuilder::new("Database Initialization Error", err.to_string())
                        .kind(MessageDialogKind::Error)
                        .buttons(MessageDialogButtons::Ok)
                        .show();
                    return Err(err.into());
                }
            };
            app.manage(db);

            // start pcap
            let capture = match get_capture() {
                Ok(cap) => cap,
                Err(err) => {
                    MessageDialogBuilder::new("Pcap Initialization Error", &err)
                        .kind(MessageDialogKind::Error)
                        .buttons(MessageDialogButtons::Ok)
                        .show();
                    return Err(err.into());
                }
            };

            // spawn ip handler in other thread
            let handle = Arc::new(app.app_handle());
            thread::spawn(move || poll_connections(handle, capture));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_cache, fetch_connection])
        .run(tauri::generate_context!())
        .expect("failed to start main application");
}

fn about_window(handle: AppHandle) {
    WindowBuilder::new(&handle, "about", WindowUrl::App("/about".parse().unwrap()))
        .title("About Ipmap")
        .center()
        .inner_size(320.0, 240.0)
        .menu(Menu::new())
        .build()
        .expect("failed to start about window");
}

fn get_capture() -> Result<Capture<Active>, String> {
    match Device::lookup() {
        Ok(device) => match device {
            Some(device) => match device.open() {
                Ok(cap) => Ok(cap),
                Err(err) => Err(format!(
                    "Failed to capture packets on network adapter: {:?}",
                    err
                )),
            },
            None => Err("No network adapters found".to_string()),
        },
        Err(err) => Err(format!("Failed to lookup network adapters: {:?}", err)),
    }
}

fn get_db() -> kverror::Result<MicroKV> {
    let data_dir: PathBuf = dirs_next::data_dir()
        .unwrap_or(env::current_dir()?)
        .join("ipmap");
    fs::create_dir_all(&data_dir)?;

    Ok(MicroKV::open_with_base_path("locations", data_dir)?.set_auto_commit(true))
}

fn poll_connections(handle: Arc<AppHandle>, mut capture: Capture<Active>) {
    while let Ok(packet) = capture.next_packet() {
        match SlicedPacket::from_ethernet(&packet) {
            Ok(packet) if packet.ip.is_some() => {
                let ip: IpAddr = match packet.ip.unwrap() {
                    InternetSlice::Ipv4(ip, _) => IpAddr::V4(ip.source_addr()),
                    InternetSlice::Ipv6(ip, _) => IpAddr::V6(ip.source_addr()),
                };

                if !ip_rfc::global(&ip) {
                    continue;
                }

                handle.emit_all("connection", ip).unwrap();
            }
            _ => (),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct Connection {
    pub ip: String,
    pub city: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub count: u32,
}

#[tauri::command]
fn get_cache(query: String, handle: AppHandle) -> Option<Connection> {
    return match handle.state::<MicroKV>().get::<Connection>(query) {
        Ok(connection) => connection,
        Err(err) => {
            eprintln!("err accessing db: {err}");
            None
        }
    };
}

#[derive(Clone, serde::Deserialize)]
struct IpApiResponse {
    ip: String,
    city: String,
    country_name: String,
    latitude: f64,
    longitude: f64,
}

#[tauri::command]
async fn fetch_connection(query: String, handle: AppHandle) -> Result<Connection, String> {
    let resp = match ureq::get(&format!("https://ipapi.co/{query}/json/")).call() {
        Ok(resp) => resp,
        Err(err) => return Err(err.to_string()),
    };

    let conn: IpApiResponse = match resp.into_json() {
        Ok(conn) => conn,
        Err(err) => return Err(err.to_string()),
    };

    let conn = Connection {
        ip: conn.ip,
        city: conn.city,
        country: conn.country_name,
        latitude: conn.latitude,
        longitude: conn.longitude,
        count: 0,
    };

    handle
        .state::<MicroKV>()
        .put(&conn.ip, &conn)
        .map_err(|err| err.to_string())?;

    Ok(conn)
}
