use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use pcap_dyn::{Api, CaptureTimeBuffer, Connections, Device};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State, ipc::Channel};
use tauri_specta::Event;
use tokio::time;

const EMIT_FREQ: Duration = Duration::from_millis(150);

pub enum PcapState {
    Loaded {
        pcap: Api,
        version: String,
        devices: Vec<Device>,
        capture: Arc<RwLock<Option<CaptureTimeBuffer>>>,
    },
    Unavailable(String),
}

impl Default for PcapState {
    fn default() -> Self {
        let pcap = match pcap_dyn::INSTANCE.as_ref() {
            Ok(pcap) => pcap.clone(),
            Err(err) => return Self::Unavailable(err.to_string()),
        };

        let devices = match pcap.devices() {
            Ok(devices) => devices,
            Err(err) => return Self::Unavailable(err.to_string()),
        };

        Self::Loaded {
            version: pcap.lib_version(),
            pcap,
            devices,
            capture: Arc::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub enum PcapStateInfo {
    Loaded {
        version: String,
        devices: Vec<Device>,
        capture: Option<Device>,
    },
    Unavailable(String),
}

impl From<&PcapState> for PcapStateInfo {
    fn from(state: &PcapState) -> Self {
        match state {
            PcapState::Loaded {
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
            }
            PcapState::Unavailable(e) => Self::Unavailable(e.clone()),
        }
    }
}

/// Fired any time the state of loaded or selected databases are changed on the backend.
#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct PcapStateChange(PcapStateInfo);

impl PcapStateChange {
    pub fn emit(app: &AppHandle, state: &PcapState) {
        let _ = Self(state.into()).emit(app);
    }
}

#[tauri::command]
#[specta::specta]
pub async fn sync_pcap_state(state: State<'_, PcapState>) -> Result<PcapStateInfo, String> {
    Ok(PcapStateInfo::from(state.inner()))
}

#[tauri::command]
#[specta::specta]
pub async fn start_capture(
    handle: AppHandle,
    state: State<'_, PcapState>,
    device: Device,
    channel: Channel<Connections>,
) -> Result<(), String> {
    let (pcap, capture_state) = match state.inner() {
        PcapState::Loaded { pcap, capture, .. } => (pcap, capture.clone()),
        PcapState::Unavailable(e) => return Err(e.clone()),
    };

    let cap = pcap.open_capture(device).map_err(|e| e.to_string())?;
    let buf = CaptureTimeBuffer::start(cap);

    capture_state
        .write()
        .map_err(|e| e.to_string())?
        .replace(buf);

    // thread stops when the capture state is set to None (at break)
    // at stop_capture().
    tokio::spawn(async move {
        loop {
            let Some(info) = capture_state
                .read()
                .ok()
                .map(|guard| guard.as_ref().map(|p| p.connections()))
                .flatten()
            else {
                break;
            };

            channel.send(info).unwrap();

            time::sleep(EMIT_FREQ).await;
        }

        channel.send(Connections::stop()).unwrap();
        tracing::debug!("stopped emitting active connections");
    });

    PcapStateChange::emit(&handle, state.inner());

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn stop_capture(handle: AppHandle, state: State<'_, PcapState>) -> Result<(), String> {
    let capture_state = match state.inner() {
        PcapState::Loaded { capture, .. } => capture,
        PcapState::Unavailable(e) => return Err(e.clone()),
    };

    capture_state.write().map_err(|e| e.to_string())?.take();

    PcapStateChange::emit(&handle, state.inner());

    Ok(())
}
