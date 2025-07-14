use std::{collections::HashMap, net::IpAddr, time::Duration};

use base64::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;

pub const EXE_NAME: &str = "ipmap-child";

#[cfg(windows)]
pub mod windows {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

    pub fn wide_null(s: impl AsRef<OsStr>) -> Vec<u16> {
        s.as_ref().encode_wide().chain(Some(0)).collect()
    }

    pub fn pipe_name(id: u64) -> String {
        format!(r"\\.\pipe\ipmap-{id}")
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
    pub fn to_arg_string(&self) -> String {
        BASE64_STANDARD.encode(serde_json::to_string(self).expect("encode Command as json"))
    }

    pub fn from_arg_string(s: impl AsRef<[u8]>) -> Option<Self> {
        BASE64_STANDARD
            .decode(s)
            .ok()
            .and_then(|s| serde_json::from_slice(&s).ok())
    }
}

#[derive(Copy, Clone, Debug, Type, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TracerouteParams {
    pub ip: IpAddr,
    pub max_rounds: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct PcapStatus {
    pub devices: Vec<Device>,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error, Type)]
#[serde(tag = "t", content = "c")]
pub enum ChildError {
    #[error("Insufficient network permissions on ipmap-child process")]
    InsufficientPermissions,
    #[error("Libpcap loading error: {0}")]
    LibLoading(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
}

#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MovingAverageInfo {
    pub total: usize,
    pub avg_s: usize,
}

#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ConnectionInfo {
    pub up: MovingAverageInfo,
    pub down: MovingAverageInfo,
}

/// A network device, e.g. "wlp3s0".
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
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

#[derive(Default, Clone, Debug, Serialize, Deserialize, Type)]
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
