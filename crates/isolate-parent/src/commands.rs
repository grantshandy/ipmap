use std::{
    io,
    sync::{Arc, Mutex},
};

use isolate_ipc::{
    CaptureParams, Command, Connections, Device, Error, Response, Status, TracerouteParams,
    TracerouteResponse,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{ipc::Channel, State};
use tauri_specta::Event;

use crate::{ipc, state::PcapState};



#[tauri::command]
#[specta::specta]
pub fn start_capture(state: State<'_, Result<PcapState, String>>, params: CaptureParams) -> Result<(), String> {


    let (mut child, exit) =
        ipc::spawn_child_process(Command::Capture(params)).map_err(|e| e.to_string())?;



    Ok(())
}

/// Stop the current capture.
#[tauri::command]
#[specta::specta]
pub async fn stop_capture(state: State<'_, Result<PcapState, String>>) -> Result<(), String> {
    state
        .as_ref()
        .map(|s| s.capture.clone())?
        .write()
        .map_err(|e| e.to_string())?
        .take()();

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn init_pcap() -> Result<PcapStateInfo, String> {
    match ipc::call_child_process(Command::Status) {
        Ok(Response::Status(status)) => Ok(PcapStateInfo {
            version: status.version,
            devices: status.devices,
            capture: None, // No capture is active at initialization
        }),
        Ok(_) => Err("Unexpected response type".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct PcapStateInfo {
    /// The version information about the currently loaded libpcap
    version: String,
    /// The list of available network devices for capture
    devices: Vec<Device>,
    /// The currently-captured on device, if any
    capture: Option<Device>,
}

#[tauri::command]
#[specta::specta]
pub fn run_traceroute(
    params: TracerouteParams,
    progress: Channel<usize>,
) -> Result<TracerouteResponse, String> {
    let (mut child, _) =
        ipc::spawn_child_process(Command::Traceroute(params)).map_err(|e| e.to_string())?;

    while let Some(message) = child.next() {
        match message {
            Ok(Response::Traceroute(resp)) => return Ok(resp),
            Ok(Response::TracerouteProgress(round)) => {
                let _ = progress.send(round);
            }
            Ok(_) => {
                return Err("Child process returned unexpected type".to_string());
            }
            Err(e) => return Err(e.to_string()),
        }
    }

    Err("Child process terminated unexpectedly".to_string())
}
