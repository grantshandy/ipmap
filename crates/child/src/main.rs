#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, net::IpAddr, panic, sync::mpsc, thread};

use child_ipc::{
    Error, ErrorKind, IpcService, PcapStatus, Response, RunCapture, RunTraceroute,
    ipc::{self},
};
use pcap_dyn::{Api, buf::CaptureTimeBuffer};

fn main() {
    Service::execute();
}

pub struct Service;

impl IpcService for Service {
    fn get_pcap_status() -> Result<PcapStatus, Error> {
        let api = Api::init().map_err(|e| Error::message(ErrorKind::LibLoading, e.to_string()))?;

        #[cfg(target_os = "linux")]
        if !Self::has_net_raw()? {
            return Err(Error::insufficient_permissions(
                env::current_exe().unwrap_or(child_ipc::EXE_NAME.into()),
            ));
        }

        let devices = api.devices().map_err(|e| Error::runtime(e.to_string()))?;

        Ok(PcapStatus {
            devices,
            version: api.lib_version(),
        })
    }

    fn start_capture(parent: &ipc::Parent, params: RunCapture) -> ! {
        let api = match Api::init() {
            Ok(api) => api,
            Err(err) => {
                ipc::exit_with_error(
                    parent,
                    Error::message(ErrorKind::LibLoading, err.to_string()),
                );
            }
        };

        let cap = match api.open_capture(params.device) {
            Ok(capture) => capture,
            Err(e) => ipc::exit_with_error(parent, Error::runtime(e.to_string())),
        };
        let buf = CaptureTimeBuffer::start(cap, params.connection_timeout);

        loop {
            ipc::send_response(parent, Ok(Response::CaptureSample(buf.connections())));
            std::thread::sleep(params.report_frequency);
        }
    }

    fn traceroute(parent: &ipc::Parent, params: RunTraceroute) -> Result<Vec<Vec<IpAddr>>, Error> {
        ipc::send_response(parent, Ok(Response::Progress(0)));

        // I've found that trippy_core sometimes panics, so you have to do this B.S.
        // Also, [Parent] doesn't go across the catch_unwind boundary so it makes it
        // much worse.

        enum ThreadMessage {
            Progress(usize),
            Result(trippy_core::State),
            Error(String),
        }

        let (tx, rx) = mpsc::channel::<ThreadMessage>();

        thread::spawn(move || {
            let panic_error = panic::catch_unwind(|| {
                let tracer = match trippy_core::Builder::new(params.ip)
                    .max_rounds(Some(params.max_rounds))
                    .build()
                {
                    Ok(tracer) => tracer,
                    Err(error) => {
                        tx.send(ThreadMessage::Error(format!(
                            "Error building traceroute: {error}"
                        )))
                        .unwrap();
                        return;
                    }
                };

                if let Err(error) = tracer.run_with(|round| {
                    let round: Option<usize> = round
                        .probes
                        .iter()
                        .filter_map(|status| match status {
                            trippy_core::ProbeStatus::Awaited(a) => Some(a.round.0),
                            trippy_core::ProbeStatus::Complete(c) => Some(c.round.0),
                            _ => None,
                        })
                        .max();

                    if let Some(round) = round {
                        tx.send(ThreadMessage::Progress(round)).unwrap();
                    }
                }) {
                    tx.send(ThreadMessage::Error(format!(
                        "Error running traceroute: {error}"
                    )))
                    .unwrap();
                    return;
                }

                tx.send(ThreadMessage::Result(tracer.snapshot())).unwrap();
            });

            if panic_error.is_err() {
                tx.send(ThreadMessage::Error("Traceroute panicked".into()))
                    .unwrap();
            }
        });

        let snapshot = loop {
            match rx.recv() {
                Ok(ThreadMessage::Result(r)) => break r,
                Ok(ThreadMessage::Progress(p)) => {
                    ipc::send_response(&parent, Ok(Response::Progress(p)))
                }
                Ok(ThreadMessage::Error(e)) => return Err(Error::runtime(e)),
                Err(e) => {
                    return Err(Error::runtime(format!("Traceroute thread closed: {e}")));
                }
            }
        };

        let hops = snapshot
            .hops()
            .iter()
            .map(|h| {
                h.addrs()
                    .copied()
                    .filter(ip_rfc::global)
                    .collect::<Vec<IpAddr>>()
            })
            .collect::<Vec<Vec<IpAddr>>>();

        Ok(hops)
    }

    fn has_net_raw() -> Result<bool, Error> {
        trippy_privilege::Privilege::discover()
            .map(|p| p.has_privileges())
            .map_err(|e| Error::runtime(e.to_string()))
    }
}
