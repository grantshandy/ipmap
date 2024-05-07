#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    error::Error,
    net::IpAddr,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread,
};

use etherparse::{NetHeaders, PacketHeaders};
use pcap::{Active, Capture, Inactive};
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

    if let Some(prefered) = &prefered {
        out.push(Device {
            name: prefered.name.clone(),
            desc: prefered.desc.clone(),
            prefered: true,
        });
    }

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

    Ok(out)
}

#[derive(serde::Deserialize, Clone)]
struct ChangeDevicePayload {
    name: String,
}

fn start_listening(app: &mut App) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<Capture<Inactive>>();
    let handle = Arc::new(app.handle());

    let change_handle = handle.clone();
    app.listen_global("change_device", move |event| {
        change_device(event, change_handle.clone(), tx.clone())
    });

    thread::spawn(move || ip_update_loop(handle.clone(), rx));

    Ok(())
}

fn ip_update_loop(handle: Arc<AppHandle>, rx: Receiver<Capture<Inactive>>) {
    // get first device
    let mut cap: Capture<Active> = loop {
        match rx.recv().map(|cap| cap.open()) {
            Ok(Ok(cap)) => break cap,
            Ok(Err(err)) => emit_error(&handle, format!("failed to open device: {err}")),
            _ => (),
        };

        continue;
    };

    loop {
        // update capture device if needed
        match rx.try_recv().map(|cap| cap.open()) {
            Ok(Ok(new_cap)) => {
                println!("switched device.");
                cap = new_cap;
            }
            Ok(Err(err)) => emit_error(&handle, format!("failed to open device: {err}")),
            _ => (),
        };

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

fn change_device(event: Event, handle: Arc<AppHandle>, tx: Sender<Capture<Inactive>>) {
    let Some(payload_str) = event.payload() else {
        emit_error(&handle, "change_device must have a payload");
        return;
    };

    let name = match serde_json::from_str::<ChangeDevicePayload>(payload_str) {
        Ok(payload) => payload.name,
        Err(err) => {
            emit_error(&handle, err.to_string());
            return;
        }
    };

    let cap = match Capture::from_device(name.as_str()) {
        Ok(cap) => cap,
        Err(err) => {
            emit_error(
                &handle,
                format!("failed to get capture from device {name}: {err}"),
            );
            return;
        }
    };

    if let Err(err) = tx.send(cap) {
        emit_error(
            &handle,
            format!("failed to send over change_device channel: {err}"),
        );
    }
}

fn emit_error(handle: &AppHandle, msg: impl AsRef<str>) {
    handle.emit_all("error", msg.as_ref()).expect("emit error");
}
