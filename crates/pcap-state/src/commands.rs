use std::net::IpAddr;

use child_ipc::{CaptureParams, Command, Connections, Error, Response, TracerouteParams, ipc};
use ipgeo_state::{DbState, LookupInfo};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State, ipc::Channel};

use crate::{PcapState, PcapStateChange, PcapStateInfo};

#[tauri::command]
#[specta::specta]
pub async fn start_capture(
    app: AppHandle,
    pcap: State<'_, PcapState>,
    params: CaptureParams,
    conns: Channel<Connections>,
) -> Result<(), Error> {
    let device = params.device.clone();

    let child_path = crate::resolve_child_path(app.path()).map_err(Error::Ipc)?;

    let (child, exit) = ipc::spawn_child_iterator(child_path, Command::Capture(params))
        .map_err(|e| Error::Ipc(e.to_string()))?;

    pcap.set_capture(device, exit);

    PcapStateChange::emit(&app);

    for resp in child {
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
#[cfg_attr(windows, allow(unused_variables))]
pub async fn traceroute_enabled(app: AppHandle) -> Result<(), Error> {
    #[cfg(windows)]
    return Ok(());

    #[cfg(not(windows))]
    {
        let path = crate::resolve_child_path(app.path()).map_err(Error::Ipc)?;

        match ipc::call_child_process(path.clone(), Command::TracerouteStatus)? {
            Response::TracerouteStatus(true) => Ok(()),
            Response::TracerouteStatus(false) => Err(Error::InsufficientPermissions(path)),
            _ => Err(Error::Ipc("Unexpected response from child".to_string())),
        }
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
    let child_path = crate::resolve_child_path(app.path()).map_err(Error::Ipc)?;

    let (child, exit) = ipc::spawn_child_iterator(child_path, Command::Traceroute(params))
        .map_err(|e| Error::Ipc(e.to_string()))?;

    let exit = || exit().map_err(|e| Error::Ipc(e.to_string()));

    for message in child {
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
