use std::net::IpAddr;

use etherparse::{NetHeaders, PacketHeaders};
use pcap::{Active, Capture, PacketCodec};
use rayon::iter::{ParallelBridge, ParallelIterator};
use tauri::{async_runtime, AppHandle, Manager, Runtime, State};
use time::OffsetDateTime;
use ts_rs::TS;
use uuid::Uuid;

use crate::capture_state::{CaptureState, ConnectionInfo, DirectedPacket, PacketDirection};

#[derive(serde::Serialize, Clone, PartialEq, TS)]
#[ts(export, export_to = "../../frontend/src/bindings/")]
pub struct Device {
    name: String,
    desc: Option<String>,
    prefered: bool,
}

/// List all [Device]s available to [`libpcap`](pcap).
#[tauri::command]
pub async fn list_devices() -> Result<Vec<Device>, String> {
    let mut out: Vec<Device> = Vec::new();
    let prefered = pcap::Device::lookup().map_err(|e| e.to_string())?;

    for d in pcap::Device::list().map_err(|e| e.to_string())? {
        if d.flags.is_loopback()
            || !d.flags.is_running()
            || !d.flags.is_up()
            || d.addresses.is_empty()
        {
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

/// Starts capturing packets on the network device.
/// Returns a UUID to identify the capturing thread for later cancellation.
#[tauri::command]
pub async fn start_capturing<R: Runtime>(
    handle: AppHandle<R>,
    name: String,
) -> Result<String, String> {
    tracing::info!("capturing on {name}");

    let thread_id = Uuid::new_v4();

    let cap: Capture<Active> = Capture::from_device(name.as_str())
        .map_err(|err| format!("device name \"{name}\" is invalid: {err}"))?
        .open()
        .map_err(|err| format!("failed to get open capture on {name}: {err}"))?;

    async_runtime::spawn_blocking(move || capture_thread(handle, thread_id, cap));

    Ok(thread_id.to_string())
}

/// Cancels the capturing thread identified by the UUID given by the start_capturing function.
#[tauri::command]
pub async fn stop_capturing<R: Runtime>(
    handle: AppHandle<R>,
    thread_id: String,
) -> Result<(), String> {
    tracing::info!("queueing capture stop of {thread_id} thread");

    handle.trigger_global(&thread_id, None);

    Ok(())
}

/// Handles all incoming and outgoing connections in parallel
fn capture_thread<R: Runtime>(handle: AppHandle<R>, thread_id: Uuid, cap: Capture<Active>) {
    let (tx, rx) = crossbeam_channel::unbounded();

    // other thread told us to stop :)
    handle.listen_global(&thread_id.to_string(), move |_| {
        tx.send(()).expect("stop capture");
    });

    let state = handle.state::<CaptureState>();

    tracing::info!("resetting capture history");
    state.reset_history();

    // Hack using Result<()> for concurrent control flow in a parallel stream >:)
    cap.iter(PacketSourceCodec)
        .par_bridge()
        .try_for_each(|res| {
            if rx.try_recv().is_ok() {
                return Err(()); // break
            }

            let Ok(Some((ip, packet))) = res else {
                return Ok(()); // continue
            };

            if ip.is_ipv6() {
                tracing::warn!("unhandled ipv6 captured");
                return Ok(()); // continue
            }

            state.connection(ip, packet);

            Ok(())
        })
        .ok();

    state.reset_history();

    tracing::info!("stopped {thread_id} capture thread");
}

/// A codec for decoding pcap packets into ingoing / outgoing Connections
struct PacketSourceCodec;

impl PacketCodec for PacketSourceCodec {
    type Item = Option<(IpAddr, DirectedPacket)>;

    fn decode(&mut self, packet: pcap::Packet<'_>) -> Self::Item {
        // extract source and destination headers for packet
        let (src, dest): (IpAddr, IpAddr) =
            match PacketHeaders::from_ethernet_slice(&packet).map(|h| h.net) {
                Ok(Some(NetHeaders::Ipv4(header, _))) => {
                    (header.source.into(), header.destination.into())
                }
                Ok(Some(NetHeaders::Ipv6(header, _))) => {
                    (header.source.into(), header.destination.into())
                }
                _ => return None,
            };

        // determine incoming/outgoing packet direction
        let (direction, foreign_ip) = match (ip_rfc::global(&src), ip_rfc::global(&dest)) {
            // global source, non-global dest: incoming
            (true, false) => (PacketDirection::Incoming, src),
            // non-global source, global dest: outgoing
            (false, true) => (PacketDirection::Outgoing, dest),
            // non-global source and dest: local (irrelevant)
            (true, true) | (false, false) => return None,
        };

        Some((
            foreign_ip,
            DirectedPacket {
                time: OffsetDateTime::now_utc(),
                direction,
                size: packet.data.len(),
            },
        ))
    }
}

#[tauri::command]
pub async fn current_connections(
    state: State<'_, CaptureState>,
) -> Result<Vec<ConnectionInfo>, ()> {
    Ok(state.current())
}

#[tauri::command]
pub async fn all_connections(state: State<'_, CaptureState>) -> Result<Vec<ConnectionInfo>, ()> {
    Ok(state.info())
}
