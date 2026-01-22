use std::{
    ffi::CString,
    mem,
    net::IpAddr,
    slice,
    sync::Arc,
    thread::{self, JoinHandle},
};

use crossbeam_channel::{Receiver, Sender, bounded};
use dlopen2::wrapper::Container;
use etherparse::{NetHeaders, PacketHeaders};

use crate::{
    Device, Error,
    ffi::{self, PcapTSend, Raw, pcap_pkthdr},
};

// TODO: check if this filters out things we would have previously found.
const BPF_FILTER: &str = include_str!(concat!(env!("OUT_DIR"), "/bpf_filter"));

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
            raw.pcap_open_live(device_name.as_ptr(), 65535, 1, 1, errbuf)
        })?;

        let handle = PcapTSend(handle_ptr);

        if unsafe { raw.pcap_set_immediate_mode(handle_ptr, 1) } == Some(1) {
            tracing::warn!("Failed to set libpcap immediate mode");
        }

        unsafe {
            // TODO: remove allocation, include cstr in executable inline.
            let c_filter = CString::new(BPF_FILTER).unwrap();
            let mut bpf_program = mem::zeroed();

            // Compile the string into bytecode
            if raw.pcap_compile(handle_ptr, &mut bpf_program, c_filter.as_ptr(), 1, 0) == 0 {
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
            if let Some(packet) = Packet::from_raw(header, packet) {
                if let Err(_) = state.packet_tx.send(packet) {
                    // Channel closed, Stop the loop.
                    state.raw.pcap_breakloop(state.handle.0);
                }
            }
        }
    }
}

/// A generic representation of a packet, mainly for statistical purposes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Packet {
    pub len: usize,
    pub ip: IpAddr,
    pub direction: PacketDirection,
}

impl Packet {
    unsafe fn from_raw(header: *const pcap_pkthdr, packet: *const libc::c_uchar) -> Option<Self> {
        let header = unsafe { *header };
        let packet = unsafe { slice::from_raw_parts(packet, header.caplen as _) };

        let (src, dst) = match PacketHeaders::from_ethernet_slice(packet).map(|h| h.net) {
            Ok(Some(NetHeaders::Ipv4(h, _))) => (h.source.into(), h.destination.into()),
            Ok(Some(NetHeaders::Ipv6(h, _))) => (h.source.into(), h.destination.into()),
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
