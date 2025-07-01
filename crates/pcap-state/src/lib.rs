use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use pcap_dyn::{Api, Device};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;


const EMIT_FREQ: Duration = Duration::from_millis(150);

pub struct PcapState {
    pcap: Api,
    version: String,
    devices: Vec<Device>,
    // capture: Arc<RwLock<Option<CaptureTimeBuffer>>>,
}

type ManagedPcapState = Result<PcapState, String>;

impl PcapState {
    pub fn new() -> ManagedPcapState {
        let pcap = pcap_dyn::INSTANCE
            .as_ref()
            .map_err(|e| e.to_string())
            .cloned()?;

        if !commands::net_raw_available() {
            return Err("Insufficient Permissions".to_string());
        }

        let devices = pcap.devices().map_err(|e| e.to_string())?;

        Ok(Self {
            version: pcap.lib_version(),
            pcap,
            devices,
            capture: Arc::default(),
        })
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

impl From<&PcapState> for PcapStateInfo {
    fn from(state: &PcapState) -> Self {
        let capture = state
            .capture
            .read()
            .expect("read capture")
            .as_ref()
            .map(|p| p.cap.device.clone());

        Self {
            devices: state.devices.clone(),
            version: state.version.clone(),
            capture,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct PcapStateChange(PcapStateInfo);

impl PcapStateChange {
    pub fn emit(app: &AppHandle) {
        if let Ok(pcap) = app.state::<ManagedPcapState>().inner() {
            let _ = Self(pcap.into()).emit(app);
        }
    }
}

pub mod commands {
    use super::{
        EMIT_FREQ, ManagedPcapState, PcapState, PcapStateChange, PcapStateInfo,
        buf::{CaptureTimeBuffer, Connections},
    };
    use pcap_dyn::Device;
    use tauri::{AppHandle, State, ipc::Channel};
    use tokio::time;

    /// Gets the initial libpcap connector state, and provides a channel for all future updates.
    #[tauri::command]
    #[specta::specta]
    pub async fn init_pcap(state: State<'_, ManagedPcapState>) -> Result<PcapStateInfo, String> {
        state.as_ref().map(|s| s.into()).map_err(|e| e.to_string())
    }

    /// Starts capture on a given device, providing a connection channel for recieving statuses
    #[tauri::command]
    #[specta::specta]
    pub async fn start_capture(
        handle: AppHandle,
        state: State<'_, Result<PcapState, String>>,
        device: Device,
        connection_channel: Channel<Connections>,
    ) -> Result<(), String> {
        let state = state.as_ref()?;

        let cap = state.pcap.open_capture(device).map_err(|e| e.to_string())?;
        let buf = CaptureTimeBuffer::start(cap);

        state
            .capture
            .write()
            .map_err(|e| e.to_string())?
            .replace(buf);

        PcapStateChange::emit(&handle);

        let capture_state = state.capture.clone();

        // thread stops when the capture state is set to None (at break)
        // at stop_capture().
        tokio::spawn(async move {
            loop {
                let Some(info) = capture_state
                    .read()
                    .ok()
                    .and_then(|guard| guard.as_ref().map(|p| p.connections()))
                else {
                    break;
                };

                let _ = connection_channel.send(info);
                time::sleep(EMIT_FREQ).await;
            }

            let _ = connection_channel.send(Connections::stop());
            PcapStateChange::emit(&handle);

            tracing::debug!("stopped emitting active connections");
        });

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
            .take();

        Ok(())
    }

    /// Check to see if capture is even available
    #[tauri::command]
    #[specta::specta]
    pub fn net_raw_available() -> bool {
        #[cfg(target_os = "linux")]
        return caps::has_cap(None, caps::CapSet::Effective, caps::Capability::CAP_NET_RAW)
            .unwrap_or(false);

        #[cfg(not(target_os = "linux"))]
        return true;
    }
}
