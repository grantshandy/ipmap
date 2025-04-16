use std::{collections::HashMap, net::IpAddr, sync::RwLock, time::Duration};

use dashmap::DashMap;
use pcap_dyn::{Packet, PacketDirection};
use serde::{Deserialize, Serialize};
use specta::Type;
use time::{Duration as TimeDuration, UtcDateTime};

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(4);
const BPS_UPDATE_INTERVAL: Duration = Duration::from_secs(2);

pub struct CaptureBuffer {
    connections: DashMap<IpAddr, Connection>,
    last_flushed: RwLock<UtcDateTime>,
}

impl Default for CaptureBuffer {
    fn default() -> Self {
        CaptureBuffer {
            connections: DashMap::new(),
            last_flushed: RwLock::new(UtcDateTime::now()),
        }
    }
}

impl CaptureBuffer {
    pub fn insert(&self, packet: Packet) {
        let last_seen = UtcDateTime::now();

        if let Some(mut connection) = self.connections.get_mut(&packet.ip) {
            connection.last_seen = last_seen;

            match packet.direction {
                PacketDirection::Incoming => connection.r#in.add(packet.size),
                PacketDirection::Outgoing => connection.out.add(packet.size),
            };
        } else {
            let info = ConnectionDirectionInfo {
                size: packet.size,
                recent_size: packet.size,
                bytes_per_second: 0,
            };

            let (in_packets, out_packets) = match packet.direction {
                PacketDirection::Incoming => (info, ConnectionDirectionInfo::default()),
                PacketDirection::Outgoing => (ConnectionDirectionInfo::default(), info),
            };

            self.connections.insert(
                packet.ip,
                Connection {
                    last_seen,
                    r#in: in_packets,
                    out: out_packets,
                },
            );
        }
    }

    fn update_speeds(&self) {
        let mut last_flushed = self.last_flushed.write().expect("read last flushed");

        let now = UtcDateTime::now();

        if now - *last_flushed < BPS_UPDATE_INTERVAL {
            return;
        }

        log::debug!(
            "Updating bytes per second for {} connections",
            self.connections.len()
        );

        let secs_since_last_update = (now - last_flushed.clone()) / TimeDuration::SECOND;

        *last_flushed = now;
        drop(last_flushed);

        for mut kv in self.connections.iter_mut() {
            let connection = kv.value_mut();
            connection
                .r#in
                .update_bytes_per_second(secs_since_last_update);
            connection
                .out
                .update_bytes_per_second(secs_since_last_update);
        }
    }

    pub fn active(&self) -> HashMap<IpAddr, ConnectionInfo> {
        self.update_speeds();

        let now = UtcDateTime::now();

        self.connections
            .iter()
            .filter(|kv| now - kv.value().last_seen <= CONNECTION_TIMEOUT)
            .map(|kv| (*kv.key(), kv.value().into()))
            .collect()
    }

    pub fn all(&self) -> HashMap<IpAddr, ConnectionInfo> {
        self.update_speeds();

        self.connections
            .iter()
            .map(|kv| (*kv.key(), kv.value().into()))
            .collect()
    }
}

#[derive(Copy, Clone, Debug)]
struct Connection {
    last_seen: UtcDateTime,
    r#in: ConnectionDirectionInfo,
    out: ConnectionDirectionInfo,
}

#[derive(Copy, Clone, Debug, Default)]
struct ConnectionDirectionInfo {
    size: usize,
    recent_size: usize,
    bytes_per_second: usize,
}

impl ConnectionDirectionInfo {
    pub fn add(&mut self, size: usize) {
        self.size += size;
        self.recent_size += size;
    }

    pub fn update_bytes_per_second(&mut self, secs_since_last_flush: f64) {
        self.bytes_per_second = (self.recent_size as f64 / secs_since_last_flush) as usize;
        self.recent_size = 0;
    }
}

#[derive(Copy, Clone, Debug, Type, Serialize, Deserialize)]
pub struct ConnectionInfo {
    r#in: usize,
    in_bps: usize,
    out: usize,
    out_bps: usize,
}

impl From<&Connection> for ConnectionInfo {
    fn from(connection: &Connection) -> Self {
        ConnectionInfo {
            r#in: connection.r#in.size,
            in_bps: connection.r#in.bytes_per_second,
            out: connection.out.size,
            out_bps: connection.out.bytes_per_second,
        }
    }
}
