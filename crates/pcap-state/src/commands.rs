use std::net::IpAddr;

use child_ipc::{Command, Connections, Error, ErrorKind, Response, RunCapture, RunTraceroute, ipc};
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
    params: RunCapture,
    conns: Channel<Connections>,
) -> Result<(), Error> {
    let device = params.device.clone();

    let child_path = crate::resolve_child_path(app.path())?;
    let (child, exit) = ipc::spawn_child_process(child_path, Command::Capture(params))?;

    pcap.set_capture(device, exit);
    PcapStateChange::emit(&app);

    while let Ok(resp) = child.recv() {
        match resp {
            Ok(Response::CaptureSample(c)) => {
                let _ = conns.send(c);
            }
            Ok(_) => {
                pcap.stop_capture();
                return Err(Error::basic(ErrorKind::UnexpectedType));
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
pub async fn stop_capture(pcap: State<'_, PcapState>) -> Result<(), Error> {
    match pcap.stop_capture() {
        Some(Err(e)) => Err(Error::from(e)),
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
pub fn print_error(error: Error) -> String {
    error.to_string()
}

#[tauri::command]
#[specta::specta]
#[cfg_attr(windows, allow(unused_variables))]
pub async fn traceroute_enabled(app: AppHandle) -> Result<(), Error> {
    #[cfg(windows)]
    return Ok(());

    #[cfg(not(windows))]
    {
        let path = crate::resolve_child_path(app.path())?;

        match ipc::call_child_process(path.clone(), Command::TracerouteStatus)? {
            Response::TraceStatus(true) => Ok(()),
            Response::TraceStatus(false) => Err(Error::message(
                ErrorKind::InsufficientPermissions,
                path.display().to_string(),
            )),
            _ => Err(Error::basic(ErrorKind::UnexpectedType)),
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn run_traceroute(
    app: AppHandle,
    db: State<'_, DbState>,
    params: RunTraceroute,
    progress: Channel<usize>,
) -> Result<Vec<Hop>, Error> {
    let child_path = crate::resolve_child_path(app.path())?;

    let (child, exit) = ipc::spawn_child_process(child_path, Command::Traceroute(params))?;

    let exit = || exit().map_err(|e| Error::from(e));

    while let Ok(msg) = child.recv() {
        match msg {
            Ok(Response::Progress(round)) => {
                let _ = progress.send(round);
            }
            Ok(Response::Traceroute(resp)) => {
                let hops = resp
                    .into_iter()
                    .map(|ips| Hop::new(ips, db.clone()))
                    .collect();

                exit()?;
                return Ok(hops);
            }
            Ok(_) => {
                exit()?;
                return Err(Error::basic(ErrorKind::UnexpectedType));
            }
            Err(e) => {
                exit()?;
                return Err(e);
            }
        }
    }

    exit()?;
    Err(Error::basic(ErrorKind::TerminatedUnexpectedly))
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
