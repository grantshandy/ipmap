use std::{
    collections::{HashMap, VecDeque},
    net::IpAddr,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use crate::{Capture, Packet, PacketDirection};
use child_ipc::{Connection, Connections, Throughput};
use dashmap::DashMap;

/// TODO: make changable?
const RATE_WINDOW_SECS: f64 = Duration::from_secs(1).as_secs_f64();

/// Collects the state of the capture over time in a buffer.
pub struct CaptureTrafficMonitor {
    pub cap: Capture,
    buf: Arc<TrafficMonitor>,
}

impl CaptureTrafficMonitor {
    // Start the capture and collect the state on another thread.
    pub fn start(mut cap: Capture, connection_timeout: Duration) -> Self {
        let packet_recv = cap.start();

        let buf = Arc::new(TrafficMonitor::new(connection_timeout));

        // Thread exits when the capture sender is dropped,
        // so this exits when Self is dropped.
        let buf_add = buf.clone();
        thread::spawn(move || {
            for packet in packet_recv {
                buf_add.add_sample(packet);
            }
        });

        Self { cap, buf }
    }

    pub fn connections(&self) -> Connections {
        self.buf.connections()
    }
}

struct TrafficMonitor {
    /// The current "active" connections
    active: DashMap<IpAddr, ActiveConnectionTracker>,
    /// Connections that are not currently active, just stores the total number of bytes.
    inactive: DashMap<IpAddr, InactiveConnectionRecord>,
    /// The amount of time since the last packet before a connection is labeled as inactive.
    connection_timeout: Duration,
}

impl TrafficMonitor {
    pub fn new(connection_timeout: Duration) -> Self {
        Self {
            active: DashMap::default(),
            inactive: DashMap::default(),
            connection_timeout,
        }
    }

    /// Add a single packet sample
    pub fn add_sample(&self, packet: Packet) {
        self.active
            .entry(packet.ip)
            .or_insert(
                // try to recover inactive connection to keep totals
                self.inactive
                    .remove(&packet.ip)
                    .map(|(_, v)| ActiveConnectionTracker::from(v))
                    .unwrap_or_default(),
            )
            .add_sample(packet);
    }

    /// Return the current state of the network traffic
    pub fn connections(&self) -> Connections {
        let mut updates = HashMap::new();
        let mut ended = Vec::new();

        let now = Instant::now();

        // fill updates, started, and ended
        for mut kv in self.active.iter_mut() {
            let ac = kv.value_mut();
            let info = ac.info(now, self.connection_timeout);

            if info.inactive() {
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

        let session = self.session(&updates);

        Connections { updates, session }
    }

    /// A single [ConnectionInfo] that represents the total bytes and throughput of the entire session since it was created.
    fn session(&self, active_info: &HashMap<IpAddr, Connection>) -> Connection {
        let (active_up_total, up_s, active_down_total, down_s) =
            active_info
                .iter()
                .fold((0, 0.0, 0, 0.0), |(a, b, c, d), (_, i)| {
                    (
                        a + i.up.total,   // active_up_total
                        b + i.up.avg_s,   // up_s
                        c + i.down.total, // active_down_total
                        d + i.down.avg_s, // down_s
                    )
                });

        let (inactive_up_total, inactive_down_total) =
            self.inactive.iter().fold((0, 0), |(x, y), kv| {
                let info = kv.value();
                (x + info.up_total, y + info.down_total)
            });

        Connection {
            up: Throughput {
                total: active_up_total + inactive_up_total,
                avg_s: up_s,
            },
            down: Throughput {
                total: active_down_total + inactive_down_total,
                avg_s: down_s,
            },
        }
    }
}

/// The current state of an active connection.
#[derive(Default)]
struct ActiveConnectionTracker {
    up: ThroughputTracker,
    down: ThroughputTracker,
}

impl ActiveConnectionTracker {
    pub fn add_sample(&mut self, packet: Packet) {
        match packet.direction {
            PacketDirection::Up => self.up.add_sample(packet.len),
            PacketDirection::Down => self.down.add_sample(packet.len),
        }
    }

    pub fn info(&mut self, now: Instant, connection_timeout: Duration) -> Connection {
        Connection {
            up: self.up.info(now, connection_timeout),
            down: self.down.info(now, connection_timeout),
        }
    }
}

#[derive(Default)]
struct ThroughputTracker {
    /// Ordered samples of byte sizes, the back being the newest sample.
    data: VecDeque<(usize, Instant)>,
    /// Total amount of bytes sent since the capture session started.
    /// This is recovered from InactiveConnectionRecord.
    total_bytes: usize,
    /// Sum of bytes added between 'connection_timeout' and now.
    current_window_sum: usize,
}

impl ThroughputTracker {
    /// Add a new sample (byte count)
    pub fn add_sample(&mut self, bytes: usize) {
        self.total_bytes += bytes;
        self.current_window_sum += bytes;
        self.data.push_back((bytes, Instant::now()));
    }

    /// Remove the dead samples and return information about the throughput.
    pub fn info(&mut self, now: Instant, connection_timeout: Duration) -> Throughput {
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

        Throughput {
            total: self.total_bytes,
            avg_s,
        }
    }
}

/// Representation of a connection that is not currently active.
/// A "lossy subset" of ActiveConnectionTracker
struct InactiveConnectionRecord {
    up_total: usize,
    down_total: usize,
}

impl From<ActiveConnectionTracker> for InactiveConnectionRecord {
    fn from(value: ActiveConnectionTracker) -> Self {
        Self {
            up_total: value.up.total_bytes,
            down_total: value.down.total_bytes,
        }
    }
}

impl From<InactiveConnectionRecord> for ActiveConnectionTracker {
    fn from(value: InactiveConnectionRecord) -> ActiveConnectionTracker {
        ActiveConnectionTracker {
            up: ThroughputTracker {
                total_bytes: value.up_total,
                ..Default::default()
            },
            down: ThroughputTracker {
                total_bytes: value.down_total,
                ..Default::default()
            },
        }
    }
}
