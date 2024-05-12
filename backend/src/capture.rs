use std::{
    net::{IpAddr, Ipv4Addr},
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
use dashmap::DashSet;
use etherparse::{NetHeaders, PacketHeaders};
use pcap::{Active, Capture, PacketCodec};
use rayon::iter::{ParallelBridge, ParallelIterator};
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

struct PacketSourceCodec;

impl PacketCodec for PacketSourceCodec {
    type Item = Option<IpAddr>;

    fn decode(&mut self, packet: pcap::Packet<'_>) -> Self::Item {
        match PacketHeaders::from_ethernet_slice(&packet).map(|h| h.net) {
            Ok(Some(NetHeaders::Ipv4(header, _))) => Some(IpAddr::from(header.source)),
            Ok(Some(NetHeaders::Ipv6(header, _))) => Some(IpAddr::from(header.source)),
            _ => None,
        }
    }
}

#[tauri::command]
pub async fn start_capturing<R: Runtime>(handle: AppHandle<R>, name: String) -> Result<(), String> {
    tracing::info!("capturing on {name}");

    let cap: Capture<Active> = Capture::from_device(name.as_str())
        .map_err(|err| format!("device name \"{name}\" is invalid: {err}"))?
        .open()
        .map_err(|err| format!("failed to get open capture on {name}: {err}"))?;

    let should_stop = Arc::new(AtomicBool::new(false));

    let cancel_stop = should_stop.clone();
    handle.listen_global(BASE64_ENGINE.encode(&name).replace("=", ""), move |_| {
        cancel_stop.store(true, Ordering::SeqCst)
    });

    let connections: DashSet<Ipv4Addr> = DashSet::new();

    cap
        .iter(PacketSourceCodec)
        .par_bridge()
        .try_for_each(|packet| {
            if should_stop.load(Ordering::SeqCst) {
                return Err(());
            }

            let source = match packet {
                Ok(Some(IpAddr::V4(ip))) => ip,
                Ok(Some(IpAddr::V6(_))) => {
                    tracing::warn!("unhandled ipv6 connection");
                    return Ok(());
                },
                Ok(_) => return Ok(()),
                Err(_) => return Ok(()),
            };

            if connections.insert(source) && ip_rfc::global_v4(&source) {
                handle
                    .emit_all("new_connection", source)
                    .expect("emit new_connection");
            }

            Ok(())
        })
        .ok();

    tracing::info!("stopping {name}");

    Ok(())
}

#[tauri::command]
pub async fn stop_capturing<R: Runtime>(handle: AppHandle<R>, name: String) -> Result<(), String> {
    tracing::info!("queueing capture stop of {name}");

    handle.trigger_global(&BASE64_ENGINE.encode(&name).replace("=", ""), None);

    Ok(())
}
