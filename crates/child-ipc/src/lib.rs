use std::{collections::HashMap, net::IpAddr, time::Duration};

use serde::{Deserialize, Serialize};

#[cfg(any(feature = "parent", feature = "child"))]
pub mod ipc;

mod error;
pub use error::{Error, ErrorKind};

pub const EXE_NAME: &str = "ipmap-child";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Command {
    /// => Response::PcapStatus
    PcapStatus,
    /// => iters infinitely Response::CaptureSample
    Capture(RunCapture),
    /// => Response::TraceStatus
    TracerouteStatus,
    /// => iters Response::Progress then returns Response::Traceroute
    Traceroute(RunTraceroute),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    Connected,
    PcapStatus(PcapStatus),
    CaptureSample(Connections),
    TraceStatus(bool),
    Progress(usize),
    Traceroute(Vec<Vec<IpAddr>>),
}

#[cfg(feature = "child")]
pub trait IpcService {
    fn execute() {
        let (command, parent) = ipc::get_command();

        let response = match command {
            Command::PcapStatus => Self::get_pcap_status().map(Response::PcapStatus),
            Command::Capture(p) => Self::start_capture(&parent, p),
            Command::TracerouteStatus => Self::has_net_raw().map(Response::TraceStatus),
            Command::Traceroute(p) => Self::traceroute(&parent, p).map(Response::Traceroute),
        };

        ipc::send_response(&parent, response);
    }

    fn get_pcap_status() -> Result<PcapStatus, Error>;

    fn start_capture(parent: &ipc::Parent, params: RunCapture) -> !;

    fn traceroute(parent: &ipc::Parent, params: RunTraceroute) -> Result<Vec<Vec<IpAddr>>, Error>;

    fn has_net_raw() -> Result<bool, Error>;
}

impl Command {
    #[cfg(all(windows, feature = "parent"))]
    #[allow(clippy::match_like_matches_macro)]
    pub fn needs_admin(&self) -> bool {
        match self {
            Command::Traceroute(_) => true,
            _ => false,
        }
    }
}

/// A network device, e.g. "wlp3s0".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "parent", derive(specta::Type))]
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
#[cfg_attr(feature = "parent", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct Connections {
    /// The current state of all active network connections.
    pub updates: HashMap<IpAddr, ConnectionInfo>,
    /// A list of IpAddrs in updates that were just added.
    pub started: Vec<IpAddr>,
    /// A list of IpAddrs that were previously in updates but have now been removed.
    pub ended: Vec<IpAddr>,
    /// Indicates to the frontend UI that the capture session has just stopped.
    pub stopping: bool,
}

impl Connections {
    /// Indicate to the frontend UI that the capture session has just stopped.
    pub fn stop() -> Self {
        Self {
            stopping: true,
            ..Default::default()
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "parent", derive(specta::Type))]
pub struct ConnectionInfo {
    pub up: MovingAverageInfo,
    pub down: MovingAverageInfo,
}

#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "parent", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct MovingAverageInfo {
    pub total: usize,
    pub avg_s: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "parent", derive(specta::Type))]
pub struct PcapStatus {
    pub devices: Vec<Device>,
    pub version: String,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "parent", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct RunTraceroute {
    pub ip: IpAddr,
    pub max_rounds: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "parent", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct RunCapture {
    pub device: Device,
    pub connection_timeout: Duration,
    pub report_frequency: Duration,
}
