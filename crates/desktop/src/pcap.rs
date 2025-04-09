use std::{sync::RwLock, thread};

use pcap_dyn::Device;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, State};
use tauri_specta::Event;

pub struct GlobalPcapState {
    pcap: Result<pcap_dyn::Pcap<'static>, pcap_dyn::Error>,
    version: String,
    capture: RwLock<Option<(Device, pcap_dyn::Capture<'static>)>>,
}

impl Default for GlobalPcapState {
    fn default() -> Self {
        let pcap = pcap_dyn::Pcap::init();

        let version = pcap
            .as_ref()
            .ok()
            .and_then(|p| p.lib_version())
            .unwrap_or("Unknown".into());

        Self {
            pcap,
            version,
            capture: RwLock::new(None),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub enum GlobalPcapStateInfo {
    Loaded {
        version: String,
        devices: Vec<Device>,
        capturing: Option<Device>,
    },
    Unavailable(String),
}

impl From<&GlobalPcapState> for GlobalPcapStateInfo {
    fn from(state: &GlobalPcapState) -> Self {
        match state.pcap.as_ref() {
            Ok(pcap) => {
                let devices = pcap.get_devices().unwrap_or_default();

                let capturing = state
                    .capture
                    .read()
                    .ok()
                    .and_then(|guard| guard.as_ref().cloned())
                    .map(|(d, _)| d);

                Self::Loaded {
                    version: state.version.clone(),
                    devices,
                    capturing,
                }
            }
            Err(err) => Self::Unavailable(err.to_string()),
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
    let pcap = state.pcap.as_ref().map_err(|e| e.to_string())?;
    let cap = pcap.open(&device).map_err(|e| e.to_string())?;
    let recv = cap.start();

    let emit_handle = handle.clone();
    thread::spawn(move || {
        while let Ok(packet) = recv.recv() {
            println!("{packet:?}");
            NewPacket::emit(&emit_handle, packet);
        }
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
    let mut capture_state = state.capture.write().map_err(|e| e.to_string())?;

    if let Some(capture) = capture_state.as_ref() {
            capture.1.stop();
    }

    capture_state.take();
    drop(capture_state);

    PcapStateChange::emit(&handle, state.inner());

    Ok(())
}
