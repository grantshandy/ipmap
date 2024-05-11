use std::{
    collections::HashSet,
    net::Ipv4Addr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use base64::{
    alphabet,
    engine::{GeneralPurpose, GeneralPurposeConfig},
    Engine,
};
use etherparse::{NetHeaders, PacketHeaders};
use pcap::{Active, Capture};
use tauri::{AppHandle, Manager, Runtime};

const BASE64_ENGINE: GeneralPurpose =
    GeneralPurpose::new(&alphabet::URL_SAFE, GeneralPurposeConfig::new());

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
pub async fn start_capturing<R: Runtime>(handle: AppHandle<R>, name: String) -> Result<(), String> {
    tracing::info!("capturing on {name}");

    let mut cap: Capture<Active> = Capture::from_device(name.as_str())
        .map_err(|err| format!("device name \"{name}\" is invalid: {err}"))?
        .open()
        .map_err(|err| format!("failed to get open capture on {name}: {err}"))?;

    let should_stop = Arc::new(AtomicBool::new(false));

    let cancel_stop = should_stop.clone();
    handle.listen_global(BASE64_ENGINE.encode(&name).replace("=", ""), move |_| {
        cancel_stop.store(true, Ordering::SeqCst)
    });

    let mut connections: HashSet<Ipv4Addr> = HashSet::new();

    loop {
        if should_stop.load(Ordering::SeqCst) {
            tracing::info!("stopping {name}");
            break;
        }

        let Ok(packet) = cap.next_packet() else {
            continue;
        };

        let source: Ipv4Addr = match PacketHeaders::from_ethernet_slice(&packet).map(|h| h.net) {
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

    Ok(())
}

#[tauri::command]
pub async fn stop_capturing<R: Runtime>(handle: AppHandle<R>, name: String) -> Result<(), String> {
    tracing::info!("queueing capture stop of {name}");

    handle.trigger_global(&BASE64_ENGINE.encode(&name).replace("=", ""), None);

    Ok(())
}
