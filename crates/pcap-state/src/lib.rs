use std::{
    io,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use child_ipc::{
    Command, Device, EXE_NAME, Response,
    ipc::{self, Error, StopCallback},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{
    AppHandle, Manager, Runtime,
    path::{BaseDirectory, PathResolver},
};
use tauri_specta::Event;

pub mod commands;

struct CaptureSession {
    stop: StopCallback,
    device: Device,
}

pub struct PcapState {
    capture: Arc<Mutex<Option<CaptureSession>>>,
}

impl PcapState {
    pub fn new() -> Self {
        Self {
            capture: Arc::default(),
        }
    }

    pub fn stop_capture(&self) -> Option<io::Result<()>> {
        match self.capture.lock().map(|mut guard| guard.take()) {
            Ok(Some(CaptureSession { stop, .. })) => Some(stop()),
            _ => None,
        }
    }

    pub fn set_capture(&self, device: Device, stop: StopCallback) {
        self.stop_capture();

        // TODO: unwrap
        self.capture
            .lock()
            .map(|mut g| g.replace(CaptureSession { device, stop }))
            .unwrap();
    }

    pub fn info(&self, app: AppHandle) -> Result<PcapStateInfo, Error> {
        let capture: Option<Device> = self
            .capture
            .lock()
            .ok()
            .and_then(|c| c.as_ref().map(|c| c.device.clone()));

        let child = resolve_child_path(app.path()).map_err(Error::Ipc)?;

        match ipc::call_child_process(child, Command::PcapStatus)? {
            Response::PcapStatus(status) => Ok(PcapStateInfo {
                version: status.version,
                devices: status.devices,
                capture,
            }),
            _ => Err(Error::Ipc("Unexpected response type".to_string())),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct PcapStateInfo {
    /// The version information about the currently loaded libpcap
    version: String,
    /// The list of available network devices for capture
    devices: Vec<Device>,
    /// The currently-captured on device, if any
    capture: Option<Device>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
#[serde(tag = "status")]
pub enum PcapStateChange {
    Ok(PcapStateInfo),
    Err(Error),
}

impl PcapStateChange {
    pub fn emit(app: &AppHandle) {
        let info = match app.state::<PcapState>().inner().info(app.clone()) {
            Ok(info) => Self::Ok(info),
            Err(err) => Self::Err(err),
        };

        let _ = info.emit(app);
    }
}

pub(crate) fn resolve_child_path(resolver: &PathResolver<impl Runtime>) -> Result<PathBuf, String> {
    resolver
        .resolve(
            PathBuf::from("resources").join(EXE_NAME),
            BaseDirectory::Resource,
        )
        .map_err(|e| format!("{EXE_NAME} not found: {e}"))
}
