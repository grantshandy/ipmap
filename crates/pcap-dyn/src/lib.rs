#![doc = include_str!("../README.md")]

use std::{
    fmt, ptr,
    sync::{Arc, LazyLock},
};

use dlopen2::wrapper::Container;

pub mod buf;
mod cap;
mod ffi;

pub use cap::{Capture, Packet, PacketDirection};
use child_ipc::Device;
use ffi::{Raw, pcap_if_t};

/// The access point to the lazily-loaded libpcap connection.
pub static LIBRARY: LazyLock<Result<Api, dlopen2::Error>> = LazyLock::new(Api::init);

/// A handle to the libpcap library.
#[derive(Clone)]
pub struct Api {
    raw: Arc<Container<Raw>>,
}

impl Api {
    /// Try to load libpcap.
    fn init() -> Result<Self, dlopen2::Error> {
        Raw::load().map(Arc::new).map(|raw| Self { raw })
    }

    /// Returns the version of the libpcap library from `pcap_lib_version`.
    pub fn lib_version(&self) -> String {
        ffi::cstr_to_string(unsafe { self.raw.pcap_lib_version() }).unwrap_or("Unknown".into())
    }

    /// A list of all available network devices.
    pub fn devices(&self) -> Result<Vec<Device>, Error> {
        let mut all_devs: *mut pcap_if_t = ptr::null_mut();

        ffi::err_cap("pcap_findalldevs", |e| unsafe {
            self.raw.pcap_findalldevs(&mut all_devs, e)
        })?;

        let mut devices = Vec::new();

        let mut dev = all_devs;
        unsafe {
            while !dev.is_null() {
                if let Some(device) = device_from_raw(*dev) {
                    devices.push(device);
                }

                dev = (*dev).next;
            }

            self.raw.pcap_freealldevs(all_devs);
        }

        Ok(devices)
    }

    /// Start capturing packets from the given device.
    pub fn open_capture(&self, device: Device) -> Result<Capture, Error> {
        Capture::open(self.raw.clone(), device)
    }
}

unsafe fn device_from_raw(value: pcap_if_t) -> Option<Device> {
    if value.flags & ffi::PCAP_IF_LOOPBACK != 0 {
        return None;
    }

    Some(Device {
        name: ffi::cstr_to_string(value.name)?,
        description: ffi::cstr_to_string(value.description),
        ready: value.flags & (ffi::PCAP_IF_RUNNING | ffi::PCAP_IF_UP) != 0,
        wireless: value.flags & ffi::PCAP_IF_WIRELESS != 0,
    })
}

/// Errors returned from the underlying C library.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Error {
    name: &'static str,
    message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.message)
    }
}

impl std::error::Error for Error {}
