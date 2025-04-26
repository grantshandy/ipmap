use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, RwLock, mpsc::TryRecvError},
    thread,
    time::Duration,
};

use pcap_dyn::{Api, CaptureTimeBuffer, ConnectionInfo, Device};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Emitter, State};
use tauri_specta::Event;

const EMIT_FREQ: Duration = Duration::from_millis(200);

pub enum GlobalPcapState {
    Loaded {
        pcap: Api,
        version: String,
        devices: Vec<Device>,
        capture: RwLock<Option<CaptureTimeBuffer>>,
    },
    Unavailable(String),
}

impl Default for GlobalPcapState {
    fn default() -> Self {
        match pcap_dyn::INSTANCE.as_ref() {
            Ok(pcap) => Self::Loaded {
                pcap: pcap.clone(),
                version: pcap.lib_version(),
                devices: pcap.devices().unwrap_or_default(),
                capture: RwLock::new(None),
            },
            Err(err) => Self::Unavailable(err.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub enum GlobalPcapStateInfo {
    Loaded {
        version: String,
        devices: Vec<Device>,
        capture: Option<Device>,
    },
    Unavailable(String),
}

impl From<&GlobalPcapState> for GlobalPcapStateInfo {
    fn from(state: &GlobalPcapState) -> Self {
        match state {
            GlobalPcapState::Loaded {
                version,
                devices,
                capture,
                ..
            } => Self::Loaded {
                devices: devices.clone(),
                version: version.clone(),
                capture: capture
                    .try_read()
                    .ok()
                    .and_then(|guard| guard.as_ref().map(|c| c.cap.device.clone())),
            },
            GlobalPcapState::Unavailable(e) => Self::Unavailable(e.clone()),
        }
    }
}

/// Fired any time the state of loaded or selected databases are changed on the backend.
#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct ActiveConnections(HashMap<IpAddr, ConnectionInfo>);

/// Fired any time the state of loaded or selected databases are changed on the backend.
#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct PcapStateChange(GlobalPcapStateInfo);

impl PcapStateChange {
    pub fn emit(app: &AppHandle, state: &GlobalPcapState) {
        let _ = Self(GlobalPcapStateInfo::from(state)).emit(app);
    }
}

#[tauri::command]
#[specta::specta]
pub async fn pcap_state(state: State<'_, GlobalPcapState>) -> Result<GlobalPcapStateInfo, String> {
    Ok(GlobalPcapStateInfo::from(state.inner()))
}

// #[tauri::command]
// #[specta::specta]
// pub async fn all_connections(state: State<'_, GlobalPcapState>) -> Result<Option<HashMap<IpAddr, ConnectionInfo>>, String> {
//     match state.inner() {
//         GlobalPcapState::Loaded { capture, .. } => capture
//             .read()
//             .ok()
//             .map(|guard| guard.as_ref().map(|c| c.buffer.all()))
//             .ok_or("Failed to unlock capture mutex".to_string()),
//         GlobalPcapState::Unavailable(_) => Ok(None),
//     }
// }

#[tauri::command]
#[specta::specta]
pub async fn start_capture(
    handle: AppHandle,
    state: State<'_, GlobalPcapState>,
    device: Device,
) -> Result<(), String> {
    let (pcap, capture) = match state.inner() {
        GlobalPcapState::Loaded { pcap, capture, .. } => (pcap, capture),
        GlobalPcapState::Unavailable(e) => return Err(e.clone()),
    };

    let emit_handle = handle.clone();
    let cap = pcap
        .open_capture(device, EMIT_FREQ, move |info| {
            let _ = ActiveConnections(info).emit(&emit_handle);
        })
        .map_err(|e| e.to_string())?;

    capture
        .write()
        .map_err(|e| e.to_string())?
        .replace(cap);

    PcapStateChange::emit(&handle, state.inner());

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn stop_capture(
    handle: AppHandle,
    state: State<'_, GlobalPcapState>,
) -> Result<(), String> {
    let mut capture_state = match state.inner() {
        GlobalPcapState::Loaded { capture, .. } => capture.write().map_err(|e| e.to_string())?,
        GlobalPcapState::Unavailable(e) => return Err(e.clone()),
    };

    capture_state.take();
    drop(capture_state);

    tracing::info!("stopped capture");

    PcapStateChange::emit(&handle, state.inner());

    Ok(())
}
