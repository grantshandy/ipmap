use std::{io, sync::{Arc, Mutex}};

use isolate_ipc::Device;

pub struct PcapState {
    version: String,
    devices: Vec<Device>,
    capture: Arc<Mutex<Box<dyn FnOnce() -> io::Result<()>>>>,
}
