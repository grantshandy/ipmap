use std::{
    collections::HashSet,
    net::Ipv4Addr,
    sync::{Arc, Mutex, RwLock},
};

use etherparse::{NetHeaders, PacketHeaders};
use pcap::{Active, Capture};
use tauri::{AppHandle, Manager, Runtime, State};

#[derive(Default)]
pub struct CaptureState {
    should_stop: Arc<RwLock<bool>>,
    capture_name: Arc<Mutex<Option<String>>>,
}

#[derive(serde::Serialize, Clone, PartialEq)]
pub struct Device {
    name: String,
    desc: Option<String>,
    prefered: bool,
}

#[tauri::command]
pub async fn list_devices() -> Result<Vec<Device>, String> {
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

#[tauri::command]
pub async fn set_device(
    state: State<'_, CaptureState>,
    name: Option<String>,
) -> Result<(), String> {
    match &name {
        Some(name) => tracing::info!("setting device as {name}"),
        None => tracing::info!("resetting device"),
    }

    *state.capture_name.lock().expect("write capture name") = name;

    Ok(())
}

#[tauri::command]
pub async fn start_capturing<R: Runtime>(
    state: State<'_, CaptureState>,
    handle: AppHandle<R>,
) -> Result<(), String> {
    let Some(name) = state.capture_name.lock().expect("get capture name").clone() else {
        return Err("no device set".to_string());
    };

    tracing::info!("capturing on {name}");

    let mut cap: Capture<Active> = Capture::from_device(name.as_str())
        .map_err(|err| format!("device name \"{name}\" is invalid: {err}"))?
        .open()
        .map_err(|err| format!("failed to get open capture on {name}: {err}"))?;

    *state.should_stop.write().expect("read should_stop") = false;

    let mut connections: HashSet<Ipv4Addr> = HashSet::new();

    loop {
        if *state.should_stop.read().expect("read should_stop") {
            tracing::info!("stopped listening on {name}");
            *state.should_stop.write().expect("write should_stop") = false;
            break;
        }

        if let Ok(packet) = cap.next_packet() {
            let source: Ipv4Addr = match PacketHeaders::from_ethernet_slice(&packet).map(|h| h.net)
            {
                Ok(Some(NetHeaders::Ipv4(header, _))) => header.source.into(),
                // Ok(Some(NetHeaders::Ipv6(header, _))) => IpAddr::from(header.source),
                _ => continue,
            };

            if connections.insert(source) && ip_rfc::global_v4(&source) {
                handle
                    .emit_all("new_connection", source)
                    .expect("emit new_connection");
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_capturing(state: State<'_, CaptureState>) -> Result<(), String> {
    tracing::info!("queueing capture stop");
    *state.should_stop.write().expect("write should_stop") = true;

    Ok(())
}
