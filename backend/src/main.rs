#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    error::Error,
    net::IpAddr,
    sync::{Arc, RwLock},
    thread,
};

use etherparse::{NetHeaders, PacketHeaders};
use pcap::{Active, Capture};
use tauri::{App, AppHandle, Event, Manager};

fn main() {
    tauri::Builder::default()
        .setup(start_listening)
        .invoke_handler(tauri::generate_handler![device_list])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(serde::Serialize, Clone, PartialEq)]
struct Device {
    name: String,
    desc: Option<String>,
    prefered: bool,
}

#[tauri::command]
async fn device_list() -> Result<Vec<Device>, String> {
    let mut out: Vec<Device> = Vec::new();
    let prefered = pcap::Device::lookup().map_err(|e| e.to_string())?;

    for d in pcap::Device::list().map_err(|e| e.to_string())? {
        if d.flags.is_loopback() || !d.flags.is_running() {
            continue;
        }

        if let Some(prefered) = &prefered {
            if prefered.name == d.name {
                continue;
            }
        }

        out.push(Device {
            name: d.name,
            desc: d.desc,
            prefered: false,
        });
    }

    if let Some(prefered) = &prefered {
        out.insert(
            0,
            Device {
                name: prefered.name.clone(),
                desc: prefered.desc.clone(),
                prefered: true,
            },
        );
    }

    Ok(out)
}

fn start_listening(app: &mut App) -> Result<(), Box<dyn Error>> {
    let handle = Arc::new(app.handle());

    let change_handle = handle.clone();
    app.listen_global("change_device", move |event| {
        let change_handle = change_handle.clone();
        thread::spawn(move || listen_for_ips(event, change_handle));
    });

    Ok(())
}

#[derive(serde::Deserialize, Clone)]
struct ChangeDevicePayload {
    name: String,
}

fn listen_for_ips(event: Event, handle: Arc<AppHandle>) {
    let device_name = match event
        .payload()
        .map(|s| serde_json::from_str::<ChangeDevicePayload>(s))
    {
        Some(Ok(payload)) => payload.name,
        Some(Err(err)) => {
            emit_error(&handle, err.to_string());
            return;
        }
        None => {
            emit_error(&handle, "change_device must have a payload");
            return;
        }
    };

    let mut cap: Capture<Active> =
        match Capture::from_device(device_name.as_str()).map(|cap| cap.open()) {
            Ok(Ok(cap)) => cap,
            Ok(Err(err)) => {
                emit_error(
                    &handle,
                    format!("failed to get open capture {device_name}: {err}"),
                );
                return;
            }
            Err(err) => {
                emit_error(&handle, format!("failed to get {device_name}: {err}"));
                return;
            }
        };

    println!("capturing on {device_name}");

    let should_stop = Arc::new(RwLock::new(false));
    let cancel_should_stop = should_stop.clone();
    handle.listen_global("change_device", move |_| {
        *cancel_should_stop.write().expect("write should_stop") = true;
    });

    loop {
        if *should_stop.read().expect("read should_stop") {
            println!("canceled {device_name}");
            break;
        }

        if let Ok(packet) = cap.next_packet() {
            let source: IpAddr = match PacketHeaders::from_ethernet_slice(&packet).map(|h| h.net) {
                Ok(Some(NetHeaders::Ipv4(header, _))) => IpAddr::from(header.source),
                Ok(Some(NetHeaders::Ipv6(header, _))) => IpAddr::from(header.source),
                _ => continue,
            };

            if ip_rfc::global(&source) {
                handle
                    .emit_all("new_connection", source)
                    .expect("emit new_connection");
            }
        }
    }
}

fn emit_error(handle: &AppHandle, msg: impl AsRef<str>) {
    handle.emit_all("error", msg.as_ref()).expect("emit error");
}
