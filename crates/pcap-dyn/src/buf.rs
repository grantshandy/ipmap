use std::{
    collections::{HashMap, VecDeque},
    net::IpAddr,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use crate::{Capture, Packet, PacketDirection};
use child_ipc::{ConnectionDirection, ConnectionInfo, Connections, ThroughputTrackerInfo};
use dashmap::DashMap;

const RATE_WINDOW_SECS: f64 = Duration::from_secs(1).as_secs_f64();

pub struct CaptureTimeBuffer {
    pub cap: Capture,
    buf: Arc<TimeBuffer>,
}

impl CaptureTimeBuffer {
    pub fn start(cap: Capture, timeout: Duration) -> Self {
        let packet_recv = cap.start();

        let buf = Arc::new(TimeBuffer::new(timeout));

        // Thread exits when the capture sender is dropped,
        // so this exits when Self is dropped.
        let buf_cum = buf.clone();
        thread::spawn(move || {
            for packet in packet_recv {
                buf_cum.add_sample(packet);
            }
        });

        Self { cap, buf }
    }

    pub fn connections(&self) -> Connections {
        self.buf.connections()
    }
}

struct TimeBuffer {
    /// The current "active" connections
    active: DashMap<IpAddr, ActiveConnection>,

    /// Connections that are not currently active, just stores the total number of bytes.
    inactive: DashMap<IpAddr, InactiveConnection>,

    /// The amount of time since the last packet before a connection is labeled as inactive.
    timeout: Duration,
}

impl TimeBuffer {
    pub fn new(timeout: Duration) -> Self {
        Self {
            active: DashMap::default(),
            inactive: DashMap::default(),
            timeout,
        }
    }

    pub fn add_sample(&self, packet: Packet) {
        self.active
            .entry(packet.ip)
            .or_insert(
                // try to recover inactive connection to keep totals
                self.inactive
                    .remove(&packet.ip)
                    .map(|(_, v)| ActiveConnection::from(v))
                    .unwrap_or_default(),
            )
            .add_sample(packet);
    }

    pub fn connections(&self) -> Connections {
        let mut updates = HashMap::new();
        let mut started = Vec::new();
        let mut ended = Vec::new();

        // fill updates, started, and ended
        for mut kv in self.active.iter_mut() {
            let (info, status) = kv.value_mut().info(self.timeout);

            if status == ConnectionStatus::Started {
                started.push(*kv.key());
            }

            if status == ConnectionStatus::Ended {
                ended.push(*kv.key());
            } else {
                updates.insert(*kv.key(), info);
            }
        }

        // move all ended connections from active to inactive
        for ip in &ended {
            if let Some((ip, info)) = self.active.remove(ip) {
                self.inactive.insert(ip, info.into());
            };
        }

        Connections {
            session: self.session_info(&updates),
            updates,
            started,
            ended,
            stopping: false,
        }
    }

    fn session_info(&self, active_info: &HashMap<IpAddr, ConnectionInfo>) -> ConnectionInfo {
        let (active_up_total, up_s, active_down_total, down_s) = active_info
            .iter()
            .map(|(_, c)| (c.up.total, c.up.avg_s, c.down.total, c.down.avg_s))
            .fold((0, 0.0, 0, 0.0), |(w, x, y, z), (a, b, c, d)| {
                (w + a, x + b, y + c, z + d)
            });

        let (inactive_up_total, inactive_down_total) = self
            .inactive
            .iter()
            .map(|kv| (kv.value().up, kv.value().down))
            .fold((0, 0), |(x, y), (a, b)| (x + a, y + b));

        ConnectionInfo {
            up: ThroughputTrackerInfo {
                total: active_up_total + inactive_up_total,
                avg_s: up_s,
            },
            down: ThroughputTrackerInfo {
                total: active_down_total + inactive_down_total,
                avg_s: down_s,
            },
            direction: ConnectionDirection::from_speed(up_s, down_s),
        }
    }
}

#[derive(Default)]
struct ActiveConnection {
    up: ThroughputTracker,
    down: ThroughputTracker,
    status: ConnectionStatus,
}

impl ActiveConnection {
    pub fn add_sample(&mut self, packet: Packet) {
        match packet.direction {
            PacketDirection::Up => self.up.add_sample(packet.len),
            PacketDirection::Down => self.down.add_sample(packet.len),
        }
    }

    pub fn info(&mut self, connection_timeout: Duration) -> (ConnectionInfo, ConnectionStatus) {
        let up = self.up.info(connection_timeout);
        let down = self.down.info(connection_timeout);

        let info = ConnectionInfo {
            direction: ConnectionDirection::from_speed(up.avg_s, down.avg_s),
            up,
            down,
        };

        let status = if self.up.dead() && self.down.dead() {
            ConnectionStatus::Ended
        } else if self.status == ConnectionStatus::Started {
            self.status = ConnectionStatus::Active;
            ConnectionStatus::Started
        } else {
            ConnectionStatus::Active
        };

        (info, status)
    }
}

#[derive(Default, Copy, Clone, PartialEq)]
enum ConnectionStatus {
    #[default]
    Started,
    Active,
    Ended,
}

// A time-based moving average
#[derive(Default)]
struct ThroughputTracker {
    data: VecDeque<(usize, Instant)>,
    total_bytes: usize,
    current_window_sum: usize, // Sum of bytes within the current window
}

impl ThroughputTracker {
    // Add a new sample (byte count)
    // This method requires a mutable reference, so it needs a write lock.
    pub fn add_sample(&mut self, bytes: usize) {
        self.total_bytes += bytes;
        self.current_window_sum += bytes;
        self.data.push_back((bytes, Instant::now()));
    }

    pub fn info(&mut self, connection_timeout: Duration) -> ThroughputTrackerInfo {
        let now = Instant::now();

        // Since data is time-ordered, we can efficiently remove from the front.
        while let Some((old_bytes, timestamp)) = self.data.front() {
            if now - *timestamp > connection_timeout {
                self.current_window_sum -= old_bytes;
                self.data.pop_front();
            } else {
                // The front element is within the window, so all others are too.
                break;
            }
        }

        let avg_s = if self.current_window_sum == 0 {
            0.0
        } else {
            self.current_window_sum as f64 / RATE_WINDOW_SECS
        };

        ThroughputTrackerInfo {
            total: self.total_bytes,
            avg_s,
        }
    }

    pub fn dead(&self) -> bool {
        self.current_window_sum == 0
    }
}

struct InactiveConnection {
    up: usize,
    down: usize,
}

impl From<ActiveConnection> for InactiveConnection {
    fn from(value: ActiveConnection) -> Self {
        Self {
            up: value.up.total_bytes,
            down: value.down.total_bytes,
        }
    }
}

impl From<InactiveConnection> for ActiveConnection {
    fn from(value: InactiveConnection) -> ActiveConnection {
        ActiveConnection {
            up: ThroughputTracker {
                total_bytes: value.up,
                ..Default::default()
            },
            down: ThroughputTracker {
                total_bytes: value.down,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
