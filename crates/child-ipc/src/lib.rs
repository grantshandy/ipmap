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
    /// An immediate first response returned by the child, assures we're connected.
    Connected,
    PcapStatus(PcapStatus),
    CaptureSample(Connections),
    TraceStatus(bool),
    Progress(usize),
    Traceroute(Vec<Vec<IpAddr>>),
}

/// Command => Response base switching and parsing logic, to be implemented by the child.
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

/// A network device reported from libpcap, e.g. "wlp3s0".
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

/// The current state of the capture session
#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connections {
    /// The current state of all active network connections.
    pub updates: HashMap<IpAddr, Connection>,
    /// A list of IpAddrs in updates that were *just* added to updates.
    pub started: Vec<IpAddr>,
    /// A list of IpAddrs that were previously in updates but have now been removed in this iteration.
    pub ended: Vec<IpAddr>,
    /// The entire session represented as a single connection.
    pub session: Connection,
}

impl Connections {
    /// If this instance contains any meaningful information to send to the frontend.
    pub fn is_empty(&self) -> bool {
        // If there is nothing in these, self.session hasn't changed since the last iteration.
        self.ended.is_empty() && self.started.is_empty() && self.updates.is_empty()
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "parent", derive(specta::Type))]
pub struct Connection {
    pub up: Throughput,
    pub down: Throughput,
}

impl Connection {
    /// Current speed up and down per second
    pub fn throughput(&self) -> f64 {
        self.up.avg_s + self.down.avg_s
    }

    pub fn inactive(&self) -> bool {
        // TODO: should we compare with 0.0?
        self.throughput() == 0.0
    }
}

/// Current stats for a single connection direction (up or down)
#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "parent", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct Throughput {
    /// Total number of bytes since the start of the capture session
    pub total: usize,
    /// Number of bytes per second
    pub avg_s: f64,
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
