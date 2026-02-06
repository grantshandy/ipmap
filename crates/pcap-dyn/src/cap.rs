use std::{
    ffi::{CStr, CString},
    mem,
    net::IpAddr,
    slice,
    sync::Arc,
    thread::{self, JoinHandle},
};

use crossbeam_channel::{Receiver, Sender, bounded};
use dlopen2::wrapper::Container;
use etherparse::{LaxNetSlice, LaxSlicedPacket};

use crate::{
    Device, Error,
    ffi::{self, PcapTSend, Raw, pcap_pkthdr},
};

const BPF_FILTER: &CStr =
    match CStr::from_bytes_with_nul(include_bytes!(concat!(env!("OUT_DIR"), "/bpf_filter"))) {
        Ok(filter) => filter,
        Err(_) => panic!("build.rs produced invalid CStr"),
    };

/// A session currently capturing packets from the network device.
pub struct Capture {
    raw: Arc<Container<Raw>>,
    handle: PcapTSend,
    thread_handle: Option<JoinHandle<()>>,
}

impl Capture {
    pub(crate) fn open(raw: Arc<Container<Raw>>, device: Device) -> Result<Self, Error> {
        // open the device for live capture
        let device_name = CString::new(device.name.clone()).unwrap();

        let handle_ptr = ffi::err_cap("pcap_open_live", |errbuf| unsafe {
            raw.pcap_open_live(device_name.as_ptr(), 64, 1, 1, errbuf)
        })?;

        let handle = PcapTSend(handle_ptr);

        if unsafe { raw.pcap_set_immediate_mode(handle_ptr, 1) } == Some(1) {
            tracing::warn!("Failed to set libpcap immediate mode");
        }

        unsafe {
            let mut bpf_program = mem::zeroed();

            // Compile the string into bytecode
            if raw.pcap_compile(handle_ptr, &mut bpf_program, BPF_FILTER.as_ptr(), 1, 0) == 0 {
                // Load the bytecode into the kernel
                raw.pcap_setfilter(handle_ptr, &mut bpf_program);
                raw.pcap_freecode(&mut bpf_program);
            } else {
                tracing::error!("BPF compilation failed, may be slightly less efficient");
            }
        }

        Ok(Self {
            raw,
            handle,
            thread_handle: None,
        })
    }

    /// Start the capture thread.
    pub fn start(&mut self) -> Receiver<Packet> {
        let (packet_tx, packet_rx) = bounded::<Packet>(10_000);

        let mut callback_state = CallbackState {
            packet_tx,
            // We do NOT clone the raw handle here to own it.
            // We pass a copy of the pointer, but the Main thread owns the lifecycle.
            handle: self.handle.clone(),
            raw: self.raw.clone(),
        };

        let join_handle = thread::spawn(move || {
            callback_state.start_loop();
            tracing::trace!("capture thread exited");
        });

        self.thread_handle = Some(join_handle);

        packet_rx
    }

    /// Stop the capture thread.
    pub fn stop(self) {
        drop(self);
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        // 1. Break the loop
        unsafe {
            self.raw.pcap_breakloop(self.handle.0);
        }

        // 2. Wait for the loop thread to exit
        if let Some(thread) = self.thread_handle.take() {
            let _ = thread.join();
        }

        // 3. Now it is safe to close the handle
        unsafe {
            self.raw.pcap_close(self.handle.0);
        }
    }
}

struct CallbackState {
    packet_tx: Sender<Packet>,
    handle: PcapTSend,
    raw: Arc<Container<Raw>>,
}

impl CallbackState {
    /// Send all packets to self.packet_tx, blocking on the current thread.
    pub fn start_loop(&mut self) {
        let raw = self.raw.clone();

        unsafe {
            raw.pcap_loop(
                self.handle.0,
                -1, // infinite loop
                Self::callback,
                self as *mut Self as *mut _,
            );
        }
    }

    extern "C" fn callback(
        slf: *mut libc::c_uchar,
        header: *const pcap_pkthdr,
        packet: *const libc::c_uchar,
    ) {
        // Safety: 'slf' is valid as long as the capture thread is running.
        let state = unsafe { &mut *(slf as *mut Self) };

        unsafe {
            if let Some(packet) = Packet::from_raw(header, packet)
                && state.packet_tx.send(packet).is_err()
            {
                // Channel closed, Stop the loop.
                state.raw.pcap_breakloop(state.handle.0);
            }
        }
    }
}

/// A generic representation of a packet, mainly for statistical purposes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Packet {
    pub ip: IpAddr,
    pub len: usize,
    pub direction: PacketDirection,
}

impl Packet {
    unsafe fn from_raw(header: *const pcap_pkthdr, packet: *const libc::c_uchar) -> Option<Self> {
        let header = unsafe { *header };
        let packet = unsafe { slice::from_raw_parts(packet, header.caplen as _) };

        let (src, dst) = match LaxSlicedPacket::from_ethernet(packet).map(|h| h.net) {
            Ok(Some(LaxNetSlice::Ipv4(s))) => (
                s.header().source_addr().into(),
                s.header().destination_addr().into(),
            ),
            Ok(Some(LaxNetSlice::Ipv6(s))) => (
                s.header().source_addr().into(),
                s.header().destination_addr().into(),
            ),
            _ => return None,
        };

        let (direction, ip) = match (ip_rfc::global(&src), ip_rfc::global(&dst)) {
            // Traffic from a private IP (local) to a public IP (remote)
            // This is UPSTREAM traffic. We care about the destination IP.
            (false, true) => (PacketDirection::Up, dst),

            // Traffic from a public IP (remote) to a private IP (local)
            // This is DOWNSTREAM traffic. We care about the source IP.
            (true, false) => (PacketDirection::Down, src),

            // All other cases (local to local, remote to remote, etc.) are ignored.
            _ => return None,
        };

        Some(Packet {
            len: header.len as usize,
            ip,
            direction,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PacketDirection {
    Down,
    Up,
}
