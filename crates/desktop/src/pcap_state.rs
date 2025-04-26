use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use pcap_dyn::{Api, CaptureTimeBuffer, ConnectionInfo, Device};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State};
use tauri_specta::Event;

const EMIT_FREQ: Duration = Duration::from_millis(300);

pub enum GlobalPcapState {
    Loaded {
        pcap: Api,
        version: String,
        devices: Vec<Device>,
        capture: Arc<RwLock<Option<CaptureTimeBuffer>>>,
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
                capture: Arc::default(),
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
            } => {
                let capture = capture
                    .read()
                    .expect("read capture")
                    .as_ref()
                    .map(|p| p.cap.device.clone());

                Self::Loaded {
                    devices: devices.clone(),
                    version: version.clone(),
                    capture,
                }
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

#[tauri::command]
#[specta::specta]
pub async fn start_capture(
    handle: AppHandle,
    state: State<'_, GlobalPcapState>,
    device: Device,
) -> Result<(), String> {
    let (pcap, capture_state) = match state.inner() {
        GlobalPcapState::Loaded { pcap, capture, .. } => (pcap, capture.clone()),
        GlobalPcapState::Unavailable(e) => return Err(e.clone()),
    };

    let cap = pcap.open_capture(device).map_err(|e| e.to_string())?;
    let buf = CaptureTimeBuffer::start(cap);

    capture_state
        .write()
        .map_err(|e| e.to_string())?
        .replace(buf);

    let emit_handle = handle.clone();

    // thread stops when the capture state is set to None (at break)
    // at stop_capture().
    thread::spawn(move || {
        loop {
            thread::sleep(EMIT_FREQ);

            let Some(info) = capture_state
                .read()
                .ok()
                .map(|guard| guard.as_ref().map(|p| p.info()))
                .flatten()
            else {
                break;
            };

            let _ = info;

            // let _ = ActiveConnections(HashMap::default()).emit(&emit_handle);
        }

        let _ = ActiveConnections(HashMap::default()).emit(&emit_handle);
    });

    PcapStateChange::emit(&handle, state.inner());

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn stop_capture(
    handle: AppHandle,
    state: State<'_, GlobalPcapState>,
) -> Result<(), String> {
    let capture_state = match state.inner() {
        GlobalPcapState::Loaded { capture, .. } => capture,
        GlobalPcapState::Unavailable(e) => return Err(e.clone()),
    };

    capture_state.write().map_err(|e| e.to_string())?.take();

    PcapStateChange::emit(&handle, state.inner());

    Ok(())
}
