use std::{
    collections::{HashMap, VecDeque},
    net::IpAddr,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use crate::{Capture, PacketDirection, cap::Packet};
use dashmap::DashMap;

const WINDOW_DURATION: Duration = Duration::from_secs(4);

pub struct CaptureTimeBuffer {
    pub cap: Capture,
    buf: Arc<DashMap<IpAddr, Connection>>,
}

impl CaptureTimeBuffer {
    pub fn start(cap: Capture) -> Self {
        let packet_recv = cap.start();

        let buf: Arc<DashMap<IpAddr, Connection>> = Arc::default();

        // Thread exits when the capture sender is dropped,
        // so this exits when Self is dropped.
        let buf_cum = buf.clone();
        thread::spawn(move || {
            for packet in packet_recv {
                buf_cum.entry(packet.ip).or_default().add_sample(packet);
            }
        });

        Self { cap, buf }
    }

    pub fn active(&self) -> HashMap<IpAddr, ConnectionInfo> {
        self.buf
            .iter_mut()
            .map(|mut kv| (*kv.key(), kv.value_mut().info()))
            .filter(|(_, info)| info.down.avg_s + info.up.avg_s > 0)
            .collect()
    }
}

#[derive(Default)]
struct Connection {
    up: MovingAverage,
    down: MovingAverage,
}

impl Connection {
    pub fn add_sample(&mut self, packet: Packet) {
        match packet.direction {
            PacketDirection::Up => self.up.add_sample(packet.len),
            PacketDirection::Down => self.down.add_sample(packet.len),
        }
    }

    pub fn info(&mut self) -> ConnectionInfo {
        ConnectionInfo {
            up: self.up.info(),
            down: self.down.info(),
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ConnectionInfo {
    pub up: MovingAverageInfo,
    pub down: MovingAverageInfo,
}

#[derive(Copy, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MovingAverageInfo {
    pub total: usize,
    pub avg_s: usize,
}

// A time-based moving average
#[derive(Default)]
struct MovingAverage {
    data: VecDeque<(usize, Instant)>,
    total_bytes: usize,
    current_window_sum: usize, // Sum of bytes within the current window
}

impl MovingAverage {
    // Add a new sample (byte count)
    // This method requires a mutable reference, so it needs a write lock.
    pub fn add_sample(&mut self, bytes: usize) {
        self.total_bytes += bytes;
        self.current_window_sum += bytes;
        self.data.push_back((bytes, Instant::now()));
    }

    pub fn info(&mut self) -> MovingAverageInfo {
        self.clean();

        MovingAverageInfo {
            total: self.total_bytes,
            avg_s: self.get_average_bytes_per_second() as usize,
        }
    }

    /// Remove old samples outside the window
    fn clean(&mut self) {
        let now = Instant::now();

        // Since data is time-ordered, we can efficiently remove from the front.
        while let Some((old_bytes, timestamp)) = self.data.front() {
            if now - *timestamp > WINDOW_DURATION {
                self.current_window_sum -= old_bytes;
                self.data.pop_front();
            } else {
                // The front element is within the window, so all others are too.
                break;
            }
        }
    }

    /// Get the average bytes per second over the window duration
    fn get_average_bytes_per_second(&self) -> f64 {
        if self.data.len() <= 1 {
            // Need at least two points to define a meaningful duration for an average.
            return 0.0;
        }

        let oldest_timestamp = self.data.front().unwrap().1;
        let newest_timestamp = self.data.back().unwrap().1;
        let duration = newest_timestamp - oldest_timestamp;

        // Handle the case where duration is effectively zero (e.g., multiple samples at the exact same Instant)
        if duration.as_secs_f64() < f64::EPSILON {
            // If there's data, return the sum (representing a high instantaneous rate), otherwise 0.0
            return if self.current_window_sum > 0 {
                self.current_window_sum as f64
            } else {
                0.0
            };
        }

        self.current_window_sum as f64 / duration.as_secs_f64()
    }
}
