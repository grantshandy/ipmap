use std::{collections::LinkedList, net::IpAddr, time::Duration};

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use ts_rs::TS;

///! A kinda state-machine representing the state of a
///! network capture session on a particular thread, centered around [CaptureState].

const CONNECTION_LIFETIME: Duration = Duration::from_secs(3);
const DIRECTION_PERCENTAGE_MIXED_THRESHOLD: f32 = 0.3;

#[derive(Clone, Debug, Default)]
pub struct CaptureState {
    connections: DashMap<IpAddr, Connection>,
}

impl CaptureState {
    /// Adds a packet to an ip Connection's list
    ///
    /// Returns true if the ip has not been found before
    pub fn connection(&self, ip: IpAddr, packet: DirectedPacket) -> bool {
        match self.connections.get_mut(&ip) {
            Some(mut connection) => {
                connection.add(packet);
                false
            }
            None => {
                let mut connection = Connection::new(ip);
                connection.add(packet);

                self.connections.insert(ip, connection);
                true
            }
        }
    }

    /// returns information about the state of the capture session so far.
    pub fn info(&self) -> Vec<ConnectionInfo> {
        self.connections.iter_mut().map(|mut c| c.info()).collect()
    }

    /// returns all the current connections (arcs shown on the map)
    pub fn current(&self) -> Vec<ConnectionInfo> {
        self.connections
            .iter_mut()
            .map(|mut i| i.info())
            .filter(|i| i.current)
            .collect()
    }

    pub fn reset_history(&self) {
        self.connections.clear();
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum PacketDirection {
    Incoming,
    Outgoing,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct DirectedPacket {
    pub time: OffsetDateTime,       // time at time of capture
    pub direction: PacketDirection, // incoming or outgoing
    pub size: usize,                // # of bytes in payload
}

#[derive(Clone, Debug)]
pub struct Connection {
    ip: IpAddr,
    size: usize,
    // front: push new --> back: pull old
    packets: LinkedList<DirectedPacket>,
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip
    }
}

impl Connection {
    pub fn new(ip: IpAddr) -> Self {
        Self {
            ip,
            size: 0,
            packets: LinkedList::new(),
        }
    }

    /// Cleans old packets and calculates stats for ConnectionInfo
    pub fn info(&mut self) -> ConnectionInfo {
        self.remove_all_before(OffsetDateTime::now_utc() - CONNECTION_LIFETIME);

        let (incoming_sum, outgoing_sum) = self
            .packets
            .iter()
            .map(|packet| match packet.direction {
                PacketDirection::Incoming => (1, 0),
                PacketDirection::Outgoing => (0, 1),
            })
            .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));

        let curr_size = incoming_sum + outgoing_sum;

        let direction = if (incoming_sum.min(outgoing_sum) as f32 / curr_size as f32)
            < DIRECTION_PERCENTAGE_MIXED_THRESHOLD
        {
            ConnectionDirection::Mixed
        } else if incoming_sum > outgoing_sum {
            ConnectionDirection::Incoming
        } else {
            ConnectionDirection::Outgoing
        };

        ConnectionInfo {
            ip: self.ip,
            size: self.size,
            direction,
            current: curr_size != 0,
        }
    }

    /// Removes all packets from the linked list before a certain point
    fn remove_all_before(&mut self, expiry: OffsetDateTime) {
        // remove all expired packets
        while let Some(packet) = self.packets.back() {
            if packet.time <= expiry {
                self.packets.pop_back();
            } else {
                break; // when gotten to non-expired back
                       // of the stack stop pop_back()ing.
            }
        }
    }

    pub fn add(&mut self, packet: DirectedPacket) {
        self.size += packet.size;
        self.packets.push_front(packet);
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub struct ConnectionInfo {
    ip: IpAddr,
    size: usize, // # of bytes in all packets in our history
    direction: ConnectionDirection,
    current: bool, // IMPORTANT: this being true makes the arc work
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
pub enum ConnectionDirection {
    Incoming = 0,
    Mixed = 1,
    Outgoing = 2,
}
