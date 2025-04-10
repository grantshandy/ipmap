#![doc = include_str!("../README.md")]

use crate::ffi::{
    DynCFunc, PcapInner, PcapTSend, pcap_breakloop, pcap_findalldevs, pcap_loop, pcap_open_live,
    pcap_t,
};
use etherparse::{NetHeaders, PacketHeaders};
use libc::{c_char, c_int};
use std::ffi::CStr;
use std::net::IpAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, LazyLock, Mutex, mpsc};
use std::{ptr, slice, thread};

mod ffi;
mod macros;

pub const INSTANCE: LazyLock<Result<Pcap<'static>, libloading::Error>> = LazyLock::new(Pcap::init);

/// A handle to a dynamically loaded libpcap library.
#[derive(Clone)]
pub struct Pcap<'t>(Arc<PcapInner<'t>>);

impl Pcap<'_> {
    /// Load the library.
    fn init() -> Result<Self, libloading::Error> {
        (unsafe { PcapInner::init() })
            .map(Arc::new)
            .map(Self)
    }

    /// A string with information about the loaded library.
    pub fn lib_version(&self) -> Option<String> {
        cstr_to_string((self.0.pcap_lib_version)())
    }

    /// A list of available devices to capture on.
    pub fn get_devices(&self) -> Result<Vec<Device>, Error> {
        let mut all_devs: *mut ffi::pcap_if_t = ptr::null_mut();

        let handle =
            err_cap::<pcap_findalldevs, c_int>(|e| (self.0.pcap_findalldevs)(&mut all_devs, e))?;

        let mut devices = Vec::with_capacity(handle as usize);

        let mut dev = all_devs;

        unsafe {
            while !dev.is_null() {
                if let Some(device) = Device::from_raw(&*dev) {
                    devices.push(device);
                }

                dev = (*dev).next;
            }
        }

        (self.0.pcap_freealldevs)(all_devs);

        Ok(devices)
    }

    /// Open a specific device for capturing.
    pub fn open(&self, device: &Device) -> Result<Capture, Error> {
        Capture::open(&device)
    }
}

/// A network device that can be captured on
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct Device {
    pub name: String,
    pub description: Option<String>,
    pub ready: bool,
    pub wireless: bool,
}

impl Device {
    fn from_raw(value: &ffi::pcap_if_t) -> Option<Self> {
        if value.flags & ffi::PCAP_IF_LOOPBACK != 0 {
            return None;
        }

        Some(Self {
            name: cstr_to_string(value.name)?,
            description: cstr_to_string(value.description),
            ready: value.flags & (ffi::PCAP_IF_RUNNING | ffi::PCAP_IF_UP) != 0,
            wireless: value.flags & ffi::PCAP_IF_WIRELESS != 0,
        })
    }
}

/// A live handle to the network device for capturing.
#[derive(Clone)]
pub struct Capture {
    handle: PcapTSend,
    lib: Arc<PcapInner<'static>>,
    stop_signal: Arc<Mutex<()>>,
}

impl Capture {
    fn open(device: &Device) -> Result<Self, Error> {
        // unwrap: Capture can only be created from an already valid PCAP
        let Pcap(lib) = pcap_handle().expect("opening capture without a valid pcap");

        let name: *const c_char = device.name.as_ptr() as _;

        let handle = err_cap::<pcap_open_live, *mut pcap_t>(|e| {
            // TODO: sometimes returns network interface was not found
            // TODO: what are these?
            (lib.pcap_open_live)(name, 128, 1, 1000, e)
        })?;

        Ok(Self {
            handle: PcapTSend(handle),
            lib,
            stop_signal: Arc::default(),
        })
    }

    /// Start receiving packets infinitely. Spawns another thread.
    pub fn start(&self) -> Receiver<Packet> {
        let (packet_tx, packet_rx) = mpsc::channel();

        let mut handler = Handler {
            packet_tx,
            handle: self.handle,
            pcap_loop: *self.lib.pcap_loop,
            pcap_breakloop: *self.lib.pcap_breakloop,
        };

        let running = self.stop_signal.clone();

        thread::spawn(move || {
            let running = running.lock();

            handler.start_loop();

            drop(running);
        });

        packet_rx
    }

    /// Stops receiving packets, blocking until the operation completes.
    pub fn stop(&self) {
        (self.lib.pcap_breakloop)(self.handle.0);

        _ = self.stop_signal.lock().is_err();
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        self.stop();
        (self.lib.pcap_close)(self.handle.0);
    }
}

struct Handler {
    packet_tx: Sender<Packet>,
    handle: PcapTSend,
    pcap_loop: <pcap_loop as DynCFunc>::Signature,
    pcap_breakloop: <pcap_breakloop as DynCFunc>::Signature,
}

impl Handler {
    fn start_loop(&mut self) {
        (self.pcap_loop)(
            self.handle.0,
            0,
            Self::callback,
            self as *mut Self as *mut u8,
        );
    }

    extern "C" fn callback(
        slf: *mut libc::c_uchar,
        header: *const ffi::pcap_pkthdr,
        packet: *const libc::c_uchar,
    ) {
        unsafe {
            if let Some(packet) = Packet::from_raw(header, packet) {
                let slf = slf as *mut Self;

                if (*slf).packet_tx.send(packet).is_err() {
                    ((*slf).pcap_breakloop)((*slf).handle.0);
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum PacketDirection {
    Incoming,
    Outgoing,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct Packet {
    pub size: usize,
    pub ip: IpAddr,
    pub direction: PacketDirection,
}

impl Packet {
    unsafe fn from_raw(
        header: *const ffi::pcap_pkthdr,
        packet: *const libc::c_uchar,
    ) -> Option<Self> {
        let header = unsafe { *header };

        let packet = unsafe { slice::from_raw_parts(packet, header.caplen as _) };

        let (src, dst) = match PacketHeaders::from_ethernet_slice(packet).map(|h| h.net) {
            Ok(Some(NetHeaders::Ipv4(h, _))) => (h.source.into(), h.destination.into()),
            Ok(Some(NetHeaders::Ipv6(h, _))) => (h.source.into(), h.destination.into()),
            _ => return None,
        };

        let (direction, ip) = match (ip_rfc::global(&src), ip_rfc::global(&dst)) {
            (true, false) => (PacketDirection::Incoming, src),
            (false, true) => (PacketDirection::Outgoing, dst),
            _ => return None,
        };

        Some(Packet {
            size: header.len as usize,
            ip,
            direction,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to load the library: {0}")]
    LibraryLoad(#[from] libloading::Error),

    #[error("Error from pcap function {0}: {1}")]
    LibPcap(&'static str, String),
}

fn cstr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        let s = unsafe { CStr::from_ptr(ptr) };

        Some(s.to_string_lossy().to_string())
    }
}

fn err_cap<F: DynCFunc, T>(mut f: impl FnMut(*mut c_char) -> T) -> Result<T, Error> {
    // pcap/pcap.h:
    // #define PCAP_ERRBUF_SIZE 256
    let mut errbuf = [0i8; 256];

    let res = f(errbuf.as_mut_ptr());

    if errbuf[0] != 0 {
        return Err(Error::LibPcap(
            F::NAME,
            cstr_to_string(errbuf.as_mut_ptr()).unwrap_or_default(),
        ));
    }

    Ok(res)
}

fn pcap_handle() -> Option<Pcap<'static>> {
    INSTANCE.as_ref().ok().cloned()
}
