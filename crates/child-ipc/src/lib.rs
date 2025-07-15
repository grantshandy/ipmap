use std::{
    collections::HashMap,
    net::IpAddr,
    path::{Path, PathBuf},
    time::Duration,
};

#[cfg(any(feature = "parent", feature = "child"))]
use base64::prelude::*;

use serde::{Deserialize, Serialize};

#[cfg(any(feature = "parent", feature = "child"))]
pub mod ipc;

pub const EXE_NAME: &str = "ipmap-child";

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "t", content = "c")]
pub enum IpcError {
    #[error("Insufficient network permissions on pcap-child process")]
    InsufficientPermissions(PathBuf),
    #[error("Libpcap loading error: {0}")]
    LibLoading(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("IPC error: {0}")]
    Ipc(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error)]
#[serde(tag = "t", content = "c")]
pub enum ChildError {
    #[error("Insufficient network permissions on ipmap-child process")]
    InsufficientPermissions,
    #[error("Libpcap loading error: {0}")]
    LibLoading(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
}

impl ChildError {
    pub fn to_error(self, child: &Path) -> IpcError {
        match self {
            Self::InsufficientPermissions => IpcError::InsufficientPermissions(child.to_path_buf()),
            Self::LibLoading(e) => IpcError::LibLoading(e),
            Self::Runtime(e) => IpcError::Runtime(e),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Command {
    PcapStatus,
    Capture(CaptureParams),
    TracerouteStatus,
    Traceroute(TracerouteParams),
}

impl Command {
    #[cfg(feature = "parent")]
    pub fn to_arg_string(&self) -> String {
        BASE64_STANDARD.encode(serde_json::to_string(self).expect("encode Command as json"))
    }

    #[cfg(feature = "child")]
    pub fn from_arg_string(s: impl AsRef<[u8]>) -> Option<Self> {
        BASE64_STANDARD
            .decode(s)
            .ok()
            .and_then(|s| serde_json::from_slice(&s).ok())
    }

    #[cfg(all(windows, feature = "parent"))]
    #[allow(clippy::match_like_matches_macro)]
    pub fn needs_admin(&self) -> bool {
        match self {
            Command::Traceroute(_) => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TracerouteParams {
    pub ip: IpAddr,
    pub max_rounds: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct CaptureParams {
    pub device: Device,
    pub connection_timeout: Duration,
    pub report_frequency: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    PcapStatus(PcapStatus),
    CaptureSample(Connections),

    TracerouteStatus(bool),
    TracerouteResponse(TracerouteResponse),
    TracerouteProgress(usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TracerouteResponse {
    pub hops: Vec<Vec<IpAddr>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PcapStatus {
    pub devices: Vec<Device>,
    pub version: String,
}

/// A network device, e.g. "wlp3s0".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Device {
    /// Name, e.g. "wlp3s0"
    pub name: String,
    /// Note: for physical devices this is usually only on Windows.
    pub description: Option<String>,
    /// If the device is up and running.
    pub ready: bool,
    /// If the device is a wireless device.
    pub wireless: bool,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct Connections {
    pub updates: HashMap<IpAddr, ConnectionInfo>,
    pub started: Vec<IpAddr>,
    pub ended: Vec<IpAddr>,
    pub stopping: bool,
}

impl Connections {
    pub fn stop() -> Self {
        Self {
            stopping: true,
            ..Default::default()
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ConnectionInfo {
    pub up: MovingAverageInfo,
    pub down: MovingAverageInfo,
}

#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct MovingAverageInfo {
    pub total: usize,
    pub avg_s: usize,
}
