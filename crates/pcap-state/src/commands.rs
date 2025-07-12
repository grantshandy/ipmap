use std::net::IpAddr;

use child_ipc::{CaptureParams, Command, Connections, Response, TracerouteParams};
use ipgeo_state::{DbState, LookupInfo};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State, ipc::Channel};

use crate::{
    PcapState, PcapStateChange, PcapStateInfo,
    ipc::{self, Error},
};

#[tauri::command]
#[specta::specta]
pub async fn start_capture(
    app: AppHandle,
    pcap: State<'_, PcapState>,
    params: CaptureParams,
    conns: Channel<Connections>,
) -> Result<(), Error> {
    let device = params.device.clone();

    let (mut child, exit) = ipc::spawn_child_iterator(
        ipc::resolve_child_path(app.path()).map_err(Error::Ipc)?,
        Command::Capture(params),
        false,
    )
    .map_err(|e| Error::Ipc(e.to_string()))?;

    pcap.set_capture(device, exit);

    PcapStateChange::emit(&app);

    while let Some(resp) = child.next() {
        match resp {
            Ok(Response::CaptureSample(c)) => {
                let _ = conns.send(c);
            }
            Ok(_) => {
                pcap.stop_capture();
                return Err(Error::Ipc(
                    "Child process returned unexpected type".to_string(),
                ));
            }
            Err(e) => {
                pcap.stop_capture();
                return Err(e);
            }
        }
    }

    pcap.stop_capture();

    let _ = conns.send(Connections::stop());
    PcapStateChange::emit(&app);

    Ok(())
}

/// Stop the current capture.
#[tauri::command]
#[specta::specta]
pub async fn stop_capture(pcap: State<'_, PcapState>) -> Result<(), String> {
    match pcap.stop_capture() {
        Some(Err(e)) => Err(e.to_string()),
        _ => Ok(()),
    }
}

#[tauri::command]
#[specta::specta]
pub fn init_pcap(app: AppHandle, state: State<'_, PcapState>) -> Result<PcapStateInfo, Error> {
    state.info(app)
}

#[tauri::command]
#[specta::specta]
pub async fn traceroute_enabled(app: AppHandle) -> Result<bool, Error> {
    #[cfg(windows)]
    return Ok(true);

    #[cfg(not(windows))]
    match ipc::call_child_process(
        ipc::resolve_child_path(app.path()).map_err(Error::Ipc)?,
        Command::TracerouteStatus,
        false,
    )? {
        Response::TracerouteStatus(s) => Ok(s),
        _ => Err(Error::Ipc("Unexpected response from child".to_string())),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn run_traceroute(
    app: AppHandle,
    db: State<'_, DbState>,
    params: TracerouteParams,
    progress: Channel<usize>,
) -> Result<Vec<Hop>, Error> {
    let child = ipc::resolve_child_path(app.path()).map_err(Error::Ipc)?;

    let (mut child, exit) = ipc::spawn_child_iterator(child, Command::Traceroute(params), true)
        .map_err(|e| Error::Ipc(e.to_string()))?;

    let exit = || exit().map_err(|e| Error::Ipc(e.to_string()));

    while let Some(message) = child.next() {
        match message {
            Ok(Response::TracerouteProgress(round)) => {
                let _ = progress.send(round);
            }
            Ok(Response::TracerouteResponse(resp)) => {
                let hops = resp
                    .hops
                    .into_iter()
                    .map(|ips| Hop::new(ips, db.clone()))
                    .collect();

                exit()?;
                return Ok(hops);
            }
            Ok(_) => {
                exit()?;
                return Err(Error::Ipc(
                    "Child process returned unexpected type".to_string(),
                ));
            }
            Err(e) => {
                exit()?;
                return Err(e);
            }
        }
    }

    Err(Error::Ipc(
        "Child process terminated unexpectedly".to_string(),
    ))
}

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
pub struct Hop {
    ips: Vec<IpAddr>,
    loc: Option<LookupInfo>,
}

impl Hop {
    pub fn new(ips: Vec<IpAddr>, db: State<'_, DbState>) -> Self {
        let loc = ips
            .iter()
            .find_map(|ip| ipgeo_state::commands::lookup_ip(db.clone(), *ip));

        Self { ips, loc }
    }
}
