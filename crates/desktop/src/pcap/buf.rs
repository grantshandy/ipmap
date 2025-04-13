use std::net::IpAddr;

use dashmap::DashMap;
use pcap_dyn::{Packet, PacketDirection};
use serde::{Deserialize, Serialize};
use specta::Type;
use time::{Duration, UtcDateTime};
use itertools::Itertools;

pub const CAPTURE_UPDATE_FREQUENCY: Duration = Duration::seconds(2);

#[derive(Default)]
pub struct CaptureBuffer {
    connections: DashMap<IpAddr, Connection>,
}

impl CaptureBuffer {
    pub fn insert(&self, packet: Packet) {
        let last_seen = UtcDateTime::now();

        let in_packets = matches!(packet.direction, PacketDirection::Incoming) as usize;
        let out_packets = matches!(packet.direction, PacketDirection::Outgoing) as usize;

        let (in_size, out_size) = match packet.direction {
            PacketDirection::Incoming => (packet.size, 0),
            PacketDirection::Outgoing => (0, packet.size),
        };

        if let Some(mut connection) = self.connections.get_mut(&packet.ip) {
            connection.last_seen = last_seen;

            connection.in_packets += in_packets;
            connection.in_size += in_size;

            connection.out_packets += out_packets;
            connection.out_size += out_size;
        } else {
            self.connections.insert(
                packet.ip,
                Connection {
                    last_seen,
                    in_packets,
                    in_size,
                    out_size,
                    out_packets,
                },
            );
        }
    }

    pub fn active(&self) -> Vec<ConnectionInfo> {
        let now = UtcDateTime::now();

        self
            .connections
            .iter()
            .filter(|kv| now - kv.value().last_seen <= CAPTURE_UPDATE_FREQUENCY)
            .map(|kv| ConnectionInfo::from(kv.pair()))
            .sorted_by(|a, b| a.total_size().cmp(&b.total_size()).reverse())
            .collect()
    }

    pub fn all(&self) -> Vec<ConnectionInfo> {
        self
            .connections
            .iter()
            .map(|kv| ConnectionInfo::from(kv.pair()))
            .sorted_by(|a, b| a.total_size().cmp(&b.total_size()).reverse())
            .collect()
    }
}

#[derive(Copy, Clone, Debug)]
struct Connection {
    last_seen: UtcDateTime,
    in_packets: usize,
    in_size: usize,
    out_size: usize,
    out_packets: usize,
}

#[derive(Clone, Debug, Type, Serialize, Deserialize)]
pub struct ConnectionInfo {
    ip: IpAddr,
    last_seen: String,
    in_packets: usize,
    in_size: usize,
    out_size: usize,
    out_packets: usize,
}

impl ConnectionInfo {
    pub fn total_size(&self) -> usize {
        self.in_size + self.out_size
    }
}

impl From<(&IpAddr, &Connection)> for ConnectionInfo {
    fn from((ip, connection): (&IpAddr, &Connection)) -> Self {
        ConnectionInfo {
            ip: *ip,
            last_seen: connection.last_seen.to_string(),
            in_packets: connection.in_packets,
            in_size: connection.in_size,
            out_size: connection.out_size,
            out_packets: connection.out_packets,
        }
    }
}
