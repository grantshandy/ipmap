use std::{
    collections::{HashMap, VecDeque},
    net::IpAddr,
    sync::{
        Arc,
        mpsc::{RecvTimeoutError, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use dashmap::DashMap;

use crate::{Capture, PacketDirection, cap::Packet};

const WINDOW_DURATION: Duration = Duration::from_secs(4);

pub struct CaptureTimeBuffer {
    pub cap: Capture,
    emit_stop_tx: Sender<()>,
}

impl CaptureTimeBuffer {
    pub(crate) fn start(
        cap: Capture,
        emit_freq: Duration,
        emitter: impl Fn(HashMap<IpAddr, ConnectionInfo>) + Send + 'static,
    ) -> Self {
        let packet_recv = cap.start();

        let avg: Arc<DashMap<IpAddr, Connection>> = Arc::default();
        let avg_cum = avg.clone();

        thread::spawn(move || {
            for packet in packet_recv {
                avg_cum.entry(packet.ip).or_default().add_sample(packet);
            }
        });

        let (emit_stop_tx, emit_stop_rx) = std::sync::mpsc::channel();

        thread::spawn(move || {
            while let Err(RecvTimeoutError::Timeout) = emit_stop_rx.recv_timeout(emit_freq) {
                let info = avg
                    .iter_mut()
                    .map(|mut kv| (*kv.key(), kv.value_mut().info()))
                    .collect();

                emitter(info);
            }
        });

        Self { cap, emit_stop_tx }
    }

    pub fn stop(self) {
        drop(self);
    }
}

impl Drop for CaptureTimeBuffer {
    fn drop(&mut self) {
        self.emit_stop_tx.send(()).unwrap();
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
            avg_s: self.get_average_bytes_per_second(),
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

#[derive(Copy, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MovingAverageInfo {
    pub total: usize,
    pub avg_s: f64,
}
