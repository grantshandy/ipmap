use std::{
    io,
    sync::{Arc, Mutex},
};

use child_ipc::{Command, Device, Error, Response};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

pub mod commands;
pub mod ipc;

pub type StopCallback = Box<dyn FnOnce() -> io::Result<()> + Send + Sync>;

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

    pub fn info(&self) -> Result<PcapStateInfo, Error> {
        let capture: Option<Device> = self
            .capture
            .lock()
            .ok()
            .and_then(|c| c.as_ref().map(|c| c.device.clone()));

        match ipc::call_child_process(Command::PcapStatus)? {
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
        let info = match app.state::<PcapState>().inner().info() {
            Ok(info) => Self::Ok(info),
            Err(err) => Self::Err(err),
        };

        let _ = info.emit(app);
    }
}
