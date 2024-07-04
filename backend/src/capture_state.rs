use std::{
    collections::{BinaryHeap, HashMap, LinkedList},
    net::IpAddr,
    time::Duration,
};

use dashmap::DashMap;
use time::OffsetDateTime;
use uuid::Uuid;

///! A massive state-machine representing the state of a network capture session on a particular thread, centered around [CaptureState]

const CONNECTION_LIFETIME: Duration = Duration::from_secs(3);
const DIRECTION_PERCENTAGE_MIXED_THRESHOLD: f32 = 0.3;

#[derive(Clone, Debug, Default)]
pub struct CaptureState {
    thread_id: Uuid,
    connections: DashMap<IpAddr, Connection>,
}

impl CaptureState {
    pub fn new(id: Uuid) -> Self {
        Self {
            thread_id: id,
            connections: DashMap::new(),
        }
    }

    // adds a packet to an ip Connection's list
    pub fn connection(&self, ip: IpAddr, packet: DirectedPacket) {
        match self.connections.get_mut(&ip) {
            Some(mut connection) => {
                connection.packets.push_front(packet);
            }
            None => {
                let mut connection = Connection::new(ip);
                connection.add(packet);

                self.connections.insert(ip, connection);
            }
        };
    }

    /// returns information about the state of the capture session so far.
    pub fn info(&self) -> Vec<ConnectionInfo> {
        self.connections.iter_mut().map(|mut c| c.info()).collect()
    }
}


#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum PacketDirection {
    Incoming,
    Outgoing,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct DirectedPacket {
    time: OffsetDateTime,       // time at time of capture
    direction: PacketDirection, // incoming or outgoing
    size: usize,                // # of bytes in payload
}

#[derive(Clone, Debug)]
pub struct Connection {
    ip: IpAddr,
    size: usize,
    // front: [ush new --> back: pull old
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
        self.clean(OffsetDateTime::now_utc() - CONNECTION_LIFETIME);

        let (incoming_sum, outgoing_sum) = self
            .packets
            .iter()
            .map(|packet| match packet.direction {
                PacketDirection::Incoming => (1, 0),
                PacketDirection::Outgoing => (0, 1),
            })
            .fold((0, 0), |(a1, b1), (a2, b2)| (a1 + a2, b1 + b2));

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
    fn clean(&mut self, expiry: OffsetDateTime) {
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
        self.packets.push_front(packet);
        self.size += packet.size;
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ConnectionInfo {
    ip: IpAddr,
    size: usize, // # of bytes in all packets in our history
    direction: ConnectionDirection,
    current: bool, // IMPORTANT: this being true makes the arc work
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ConnectionDirection {
    Incoming,
    Mixed,
    Outgoing,
}