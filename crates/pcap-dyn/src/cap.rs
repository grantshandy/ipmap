use std::{
    ffi::CString,
    net::IpAddr,
    slice,
    sync::{
        Arc,
        mpsc::{self, Receiver, Sender},
    },
    thread,
};

use dlopen2::wrapper::Container;
use etherparse::{NetHeaders, PacketHeaders};

use crate::{
    Device, Error,
    ffi::{self, PcapTSend, Raw, pcap_pkthdr},
};

/// A session currently capturing packets from the network device.
pub struct Capture {
    /// The device being captured on.
    pub device: Device,
    raw: Arc<Container<Raw>>,
    handle: PcapTSend,

    // stop_tx: Sender<()>,
    // stop_rx: Receiver<()>,
}

impl Capture {
    pub(crate) fn open(raw: Arc<Container<Raw>>, device: Device) -> Result<Self, Error> {
        // open the device for live capture
        let device_name = CString::new(device.name.clone()).unwrap();

        // TODO: Handle Warnings:
        // pcap_open_live() returns a pcap_t * on success and NULL on
        // failure.  If NULL is returned, errbuf is filled in with an
        // appropriate error message.  errbuf may also be set to warning text
        // when pcap_open_live() succeeds; to detect this case the caller
        // should store a zero-length string in errbuf before calling
        // pcap_open_live() and display the warning to the user if errbuf is
        // no longer a zero-length string.

        let handle = ffi::err_cap("pcap_open_live", |errbuf| unsafe {
            let h = raw.pcap_open_live(device_name.as_ptr(), 128, 1, 1000, errbuf);
            // (!h.is_null()).then(|| h)
            h
        })?;


        // let (stop_tx, stop_rx) = mpsc::channel::<()>();

        Ok(Self {
            device,
            raw,
            handle: PcapTSend(handle),
            // stop_tx,
            // stop_rx,
        })
    }

    /// Start the capture thread.
    pub fn start(&self) -> Receiver<Packet> {
        let (packet_tx, packet_rx) = mpsc::channel::<Packet>();

        let mut callback_state = CallbackState {
            packet_tx,
            handle: self.handle.clone(),
            raw: self.raw.clone(),
        };

        // capture thread:
        //
        // This thread is responsible for calling pcap_loop, which will
        // block until pcap_breakloop is called in Capture::stop. Libpcap
        // doesn't like the callbacks to take very long, so we pass them into
        // an mpsc channel and return immediately.
        // let stop_tx = self.stop_tx.clone();
        thread::spawn(move || {
            // blocks until Self::stop calls pcap_breakloop
            callback_state.start_loop();

            // signal back to Self::stop that start_loop finished blocking
            // let _ = stop_tx.send(());
        });

        packet_rx
    }

    /// Stop the capture thread.
    pub fn stop(self) {
        drop(self);
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        // stop the capture thread
        unsafe {
            self.raw.pcap_breakloop(self.handle.0);
        }
        // wait until it's broken
        // _ = self.stop_rx.recv();

        // close the handle
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
    pub fn start_loop(&mut self) {
        let raw = self.raw.clone();

        unsafe {
            raw.pcap_loop(
                self.handle.0,
                0, // infinite loop
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
        unsafe {
            let Some(packet) = Packet::from_raw(header, packet) else {
                return;
            };

            let slf = slf as *mut Self;

            if (*slf).packet_tx.send(packet).is_err() {
                // If the packet_rx (in Capture) was dropped, try to break the loop if we can.
                // I'm not sure why this would happen, but this should stop the edge case where
                // Capture is dropped but the loop is still running. (no way to stop it otherwise)

                (*slf).raw.pcap_breakloop((*slf).handle.0);
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
            (false, true) => (PacketDirection::Down, src),
            (true, false) => (PacketDirection::Up, dst),
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
    Up,
    Down,
}
