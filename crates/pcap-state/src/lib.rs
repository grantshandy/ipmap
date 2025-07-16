use std::{
    io,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use child_ipc::{
    Command, Device, EXE_NAME, Error, ErrorKind, Response,
    ipc::{self, StopCallback},
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

#[derive(Default)]
pub struct PcapState {
    capture: Arc<Mutex<Option<CaptureSession>>>,
}

impl PcapState {
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

        let child = resolve_child_path(app.path())?;

        match ipc::call_child_process(child, Command::PcapStatus)? {
            Response::PcapStatus(status) => Ok(PcapStateInfo {
                version: status.version,
                devices: status.devices,
                capture,
            }),
            _ => Err(Error::basic(ErrorKind::UnexpectedType)),
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

pub(crate) fn resolve_child_path(resolver: &PathResolver<impl Runtime>) -> Result<PathBuf, Error> {
    match resolver.resolve(
        PathBuf::from("resources").join(EXE_NAME),
        BaseDirectory::Resource,
    ) {
        Ok(path) => {
            if path.try_exists().is_ok_and(|exists| exists) {
                Ok(path)
            } else {
                Err(Error::message(
                    ErrorKind::ChildNotFound,
                    format!("{path:?} doesn't exist"),
                ))
            }
        }
        Err(err) => Err(Error::message(ErrorKind::ChildNotFound, err.to_string())),
    }
}
