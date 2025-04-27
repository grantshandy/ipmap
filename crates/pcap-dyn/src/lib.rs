#![doc = include_str!("../README.md")]

use std::{
    fmt, ptr,
    sync::{Arc, LazyLock},
};

use dlopen2::wrapper::Container;

mod buf;
mod cap;
mod ffi;

pub use buf::{CaptureTimeBuffer, ConnectionInfo, MovingAverageInfo};
pub use cap::{Capture, Packet, PacketDirection};
use ffi::{Raw, pcap_if_t};

/// A static instance of the pcap library API, initialized when first used.
pub const INSTANCE: LazyLock<Result<Api, dlopen2::Error>> = LazyLock::new(Api::init);

/// A handle to the libpcap library.
#[derive(Clone)]
pub struct Api {
    raw: Arc<Container<Raw>>,
}

impl Api {
    /// Try to load libpcap.
    pub fn init() -> Result<Self, dlopen2::Error> {
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
                if let Some(device) = Device::from_raw(*dev) {
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

/// A network device, e.g. "wlp3s0".
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct Device {
    /// Name, e.g. "wlp3s0"
    pub name: String,
    /// Note: for physical devices this is usually only on Windows.
    pub description: Option<String>,
    /// If the device is up and running.
    pub ready: bool,
    /// If the device is a wireless device.
    pub wireless: bool,
}

impl Device {
    unsafe fn from_raw(value: pcap_if_t) -> Option<Self> {
        if value.flags & ffi::PCAP_IF_LOOPBACK != 0 {
            return None;
        }

        Some(Self {
            name: ffi::cstr_to_string(value.name)?,
            description: ffi::cstr_to_string(value.description),
            ready: value.flags & (ffi::PCAP_IF_RUNNING | ffi::PCAP_IF_UP) != 0,
            wireless: value.flags & ffi::PCAP_IF_WIRELESS != 0,
        })
    }
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
