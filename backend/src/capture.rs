use std::net::IpAddr;

use dashmap::DashSet;
use etherparse::{NetHeaders, PacketHeaders};
use pcap::{Active, Capture, PacketCodec};
use rayon::iter::{ParallelBridge, ParallelIterator};
use tauri::{async_runtime, AppHandle, Manager, Runtime};
use ts_rs::TS;
use uuid::Uuid;

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

/// A captured connection from the device/thread identified by `capturing_uuid`
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, serde::Serialize, TS)]
#[ts(export, export_to = "../../frontend/src/bindings/")]
pub struct Connection {
    thread_id: Uuid,
    outgoing: bool,
    ip: IpAddr,
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
    let (stop_tx, stop_rx) = crossbeam_channel::unbounded();

    handle.listen_global(&thread_id.to_string(), move |_| {
        stop_tx.send(()).expect("stop transmission");
    });

    // let connections: ExpirySet<Connection> = ExpirySet::new(CONNECTION_EXPIRY);
    let connections: DashSet<Connection> = DashSet::new();

    let codec = PacketSourceCodec(thread_id);

    // Hack using Result<()> for concurrent control flow in a parallel stream >:)
    cap.iter(codec)
        .par_bridge()
        .try_for_each(|packet| {
            if stop_rx.try_recv().is_ok() {
                return Err(()); // break
            }

            let Ok(Some(connection)) = packet else {
                return Ok(()); // continue
            };

            if connection.ip.is_ipv6() {
                tracing::warn!("unhandled ipv6 connection");
                return Ok(()); // continue
            }

            if connections.insert(connection) {
                handle
                    .emit_all("new_capture", connection)
                    .expect("emit new_connection");
            }

            Ok(())
        })
        .ok();

    tracing::info!("stopped {thread_id} capture thread");
}

/// A codec for decoding pcap packets into ingoing / outgoing Connections
struct PacketSourceCodec(Uuid);

impl PacketCodec for PacketSourceCodec {
    type Item = Option<Connection>;

    fn decode(&mut self, packet: pcap::Packet<'_>) -> Self::Item {
        let (source, destination): (IpAddr, IpAddr) =
            match PacketHeaders::from_ethernet_slice(&packet).map(|h| h.net) {
                Ok(Some(NetHeaders::Ipv4(header, _))) => {
                    (header.source.into(), header.destination.into())
                }
                Ok(Some(NetHeaders::Ipv6(header, _))) => {
                    (header.source.into(), header.destination.into())
                }
                _ => return None,
            };

        // global source, non-global dest: incoming
        // non-global source, global dest: outgoing

        match (ip_rfc::global(&source), ip_rfc::global(&destination)) {
            (true, true) | (false, false) => None,
            (true, false) => Some(Connection {
                thread_id: self.0,
                outgoing: false,
                ip: source,
            }),
            (false, true) => Some(Connection {
                thread_id: self.0,
                outgoing: true,
                ip: destination,
            }),
        }
    }
}
