use std::{collections::HashMap, net::IpAddr, time::Duration};

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Command {
    Status,
    Traceroute(TracerouteParams),
    Capture(CaptureParams),
}

#[derive(Copy, Clone, Debug, Type, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TracerouteParams {
    pub ip: IpAddr,
    pub max_rounds: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CaptureParams {
    pub device: Device,
    pub connection_timeout: Duration,
    pub report_frequency: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    Traceroute(TracerouteResponse),
    TracerouteProgress(usize),
    Status(Status),
    Connections(Connections),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TracerouteResponse {
    pub hops: Vec<Vec<IpAddr>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Status {
    pub devices: Vec<Device>,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Insufficient network permissions on isolate-child process")]
    InsufficientPermissions,
    #[error("Libpcap loading error: {0}")]
    LibLoading(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("IPC error: {0}")]
    Ipc(String),
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
}
