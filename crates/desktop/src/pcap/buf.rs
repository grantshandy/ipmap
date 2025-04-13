use std::{net::IpAddr, sync::RwLock, time::Duration};

use dashmap::DashMap;
use itertools::Itertools;
use pcap_dyn::{Packet, PacketDirection};
use serde::{Deserialize, Serialize};
use specta::Type;
use time::{Duration as TimeDuration, UtcDateTime};

pub const CAPTURE_UPDATE_FREQUENCY: Duration = Duration::from_millis(500);
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(3);

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

        let packet_direction = ConnectionDirectionInfo {
            count: 1,
            size: packet.size,
            size_since_update: packet.size,
            bytes_per_second: 0,
        };

        if let Some(mut connection) = self.connections.get_mut(&packet.ip) {
            connection.last_seen = last_seen;

            match packet.direction {
                PacketDirection::Incoming => connection.r#in.add(packet_direction),
                PacketDirection::Outgoing => connection.out.add(packet_direction),
            };
        } else {
            let (in_packets, out_packets) = match packet.direction {
                PacketDirection::Incoming => (packet_direction, ConnectionDirectionInfo::default()),
                PacketDirection::Outgoing => (ConnectionDirectionInfo::default(), packet_direction),
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

    pub fn active(&self) -> Vec<ConnectionInfo> {
        self.update_speeds();

        let now = UtcDateTime::now();

        self.connections
            .iter()
            .filter(|kv| now - kv.value().last_seen <= CONNECTION_TIMEOUT)
            .map(|kv| ConnectionInfo::from(kv.pair()))
            .sorted_by(|a, b| a.total_size().cmp(&b.total_size()).reverse())
            .collect()
    }

    pub fn all(&self) -> Vec<ConnectionInfo> {
        self.update_speeds();

        self.connections
            .iter()
            .map(|kv| ConnectionInfo::from(kv.pair()))
            .sorted_by(|a, b| a.total_size().cmp(&b.total_size()).reverse())
            .collect()
    }
}

#[derive(Copy, Clone, Debug)]
struct Connection {
    last_seen: UtcDateTime,
    r#in: ConnectionDirectionInfo,
    out: ConnectionDirectionInfo,
}

#[derive(Copy, Clone, Debug, Type, Serialize, Deserialize)]
pub struct ConnectionInfo {
    ip: IpAddr,
    r#in: ConnectionDirectionInfo,
    out: ConnectionDirectionInfo,
}

impl ConnectionInfo {
    pub fn total_size(&self) -> usize {
        self.r#in.size + self.out.size
    }
}

#[derive(Copy, Clone, Debug, Default, Type, Serialize, Deserialize)]
pub struct ConnectionDirectionInfo {
    count: usize,
    size: usize,
    size_since_update: usize,
    bytes_per_second: usize,
}

impl ConnectionDirectionInfo {
    pub fn add(&mut self, other: ConnectionDirectionInfo) {
        self.count += other.count;
        self.size += other.size;
        self.size_since_update += other.size;
    }

    pub fn update_bytes_per_second(&mut self, secs_since_last_flush: f64) {
        self.bytes_per_second = (self.size_since_update as f64 / secs_since_last_flush) as usize;
        self.size_since_update = 0;
    }
}

impl From<(&IpAddr, &Connection)> for ConnectionInfo {
    fn from((ip, connection): (&IpAddr, &Connection)) -> Self {
        ConnectionInfo {
            ip: *ip,
            r#in: connection.r#in,
            out: connection.out,
        }
    }
}
