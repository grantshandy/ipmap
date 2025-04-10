use std::{
    sync::{Arc, RwLock},
    thread,
};

use pcap_dyn::{Capture, Device, Pcap};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State};
use tauri_specta::Event;

pub enum GlobalPcapState {
    Loaded {
        pcap: Pcap<'static>,
        version: String,
        capture: RwLock<Option<(Device, Arc<Capture>)>>,
    },
    Unavailable(String),
}

impl Default for GlobalPcapState {
    fn default() -> Self {
        match pcap_dyn::INSTANCE.as_ref() {
            Ok(pcap) => Self::Loaded {
                pcap: pcap.clone(),
                version: pcap.lib_version().unwrap_or_else(|| "Unknown".into()),
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
                pcap,
                version,
                capture: capturing,
            } => Self::Loaded {
                devices: pcap.get_devices().unwrap_or_default(),
                version: version.clone(),
                capture: capturing
                    .read()
                    .ok()
                    .and_then(|guard| guard.as_ref().map(|(d, _)| d.clone())),
            },
            GlobalPcapState::Unavailable(e) => Self::Unavailable(e.clone()),
        }
    }
}

/// Fired any time the state of loaded or selected databases are changed on the backend.
#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct NewPacket(pcap_dyn::Packet);

impl NewPacket {
    pub fn emit(app: &AppHandle, packet: pcap_dyn::Packet) {
        let _ = Self(packet).emit(app);
    }
}

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
pub fn pcap_state(state: State<'_, GlobalPcapState>) -> GlobalPcapStateInfo {
    GlobalPcapStateInfo::from(state.inner())
}

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

    let cap: Arc<Capture> = pcap
        .open(&device)
        .map_err(|e| e.to_string())
        .map(Arc::new)?;

    capture
        .write()
        .map_err(|e| e.to_string())?
        .replace((device.clone(), cap.clone()));

    let recv = cap.start();

    let emit_handle = handle.clone();
    thread::spawn(move || {
        while let Ok(packet) = recv.recv() {
            NewPacket::emit(&emit_handle, packet);
        }
        tracing::info!("capture thread exited");
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
    let mut capture_state = match state.inner() {
        GlobalPcapState::Loaded { capture, .. } => capture.write().map_err(|e| e.to_string())?,
        GlobalPcapState::Unavailable(e) => return Err(e.clone()),
    };

    if let Some(capture) = capture_state.as_ref() {
        capture.1.stop();
    }

    capture_state.take();
    drop(capture_state);

    tracing::info!("finished stopping capture");

    PcapStateChange::emit(&handle, state.inner());

    Ok(())
}
