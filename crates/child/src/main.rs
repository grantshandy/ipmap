#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{net::IpAddr, thread};

use child_ipc::{
    Error, ErrorKind, IpcService, PcapStatus, Response, RunCapture, RunTraceroute,
    ipc::{self},
};
use pcap_dyn::{Api, buf::CaptureTrafficMonitor};

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
                std::env::current_exe().unwrap_or(child_ipc::EXE_NAME.into()),
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
        let buf = CaptureTrafficMonitor::start(cap, params.connection_timeout);

        let mut last_empty = false;

        loop {
            let conn = buf.connections();
            let conn_empty = conn.is_empty();

            // Send a response if:
            // 1. The buffer is not empty (always report new data).
            // 2. The buffer *just became* empty (a state change).
            if !conn_empty || !last_empty {
                ipc::send_response(parent, Ok(Response::CaptureSample(conn)));
            }

            // Update the state for the next loop iteration.
            last_empty = conn_empty;

            thread::sleep(params.report_frequency);
        }
    }

    fn traceroute(parent: &ipc::Parent, params: RunTraceroute) -> Result<Vec<Vec<IpAddr>>, Error> {
        if !Self::has_net_raw()? {
            return Err(Error::insufficient_permissions(
                std::env::current_exe().unwrap_or(child_ipc::EXE_NAME.into()),
            ));
        }

        let tracer = trippy_core::Builder::new(params.ip)
            .max_rounds(Some(params.max_rounds))
            .build()
            .map_err(|err| Error::runtime(format!("Error building traceroute: {err}")))?;

        tracer
            .run_with(|round| {
                let Some(round) = round
                    .probes
                    .iter()
                    .filter_map(|status| match status {
                        trippy_core::ProbeStatus::Awaited(a) => Some(a.round.0),
                        trippy_core::ProbeStatus::Complete(c) => Some(c.round.0),
                        _ => None,
                    })
                    .max()
                else {
                    return;
                };

                ipc::send_response(parent, Ok(Response::Progress(round)));
            })
            .map_err(|err| Error::runtime(format!("Error running traceroute: {err}")))?;

        let hops = tracer
            .snapshot()
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
