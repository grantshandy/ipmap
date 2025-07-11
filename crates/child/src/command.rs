use std::net::IpAddr;

use child_ipc::{CaptureParams, Error, PcapStatus, Response, TracerouteParams, TracerouteResponse};
use pcap_dyn::{Api, buf::CaptureTimeBuffer};

use crate::ipc;

pub fn get_pcap_status() -> Result<PcapStatus, Error> {
    let api = get_api();

    #[cfg(target_os = "linux")]
    if !has_net_raw_privileges()? {
        return Err(Error::InsufficientPermissions);
    }

    let devices = api.devices().map_err(|e| Error::Runtime(e.to_string()))?;

    Ok(PcapStatus {
        devices,
        version: api.lib_version(),
    })
}

pub fn run_capture(params: CaptureParams) -> ! {
    let api = get_api();

    let cap = match api.open_capture(params.device) {
        Ok(capture) => capture,
        Err(e) => ipc::exit_with_error(Error::Runtime(e.to_string())),
    };
    let buf = CaptureTimeBuffer::start(cap, params.connection_timeout);

    loop {
        ipc::send_response(Ok(Response::CaptureSample(buf.connections())));
        std::thread::sleep(params.report_frequency);
    }
}

pub fn run_traceroute(params: TracerouteParams) -> Result<TracerouteResponse, Error> {
    let snapshot = std::panic::catch_unwind(|| {
        let tracer = trippy_core::Builder::new(params.ip)
            .max_rounds(Some(params.max_rounds))
            .build()
            .map_err(|e| Error::Runtime(e.to_string()))?;

        tracer
            .run_with(|round| {
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
                    ipc::send_response(Ok(Response::TracerouteProgress(round)));
                }
            })
            .map_err(|e| Error::Runtime(e.to_string()))?;

        Ok(tracer.snapshot())
    })
    .map_err(|e| Error::Runtime(format!("trippy panicked: {e:?}")))??;

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

    Ok(TracerouteResponse { hops })
}

pub fn has_net_raw_privileges() -> Result<bool, Error> {
    trippy_privilege::Privilege::discover()
        .map(|p| p.has_privileges())
        .map_err(|e| Error::Runtime(e.to_string()))
}

fn get_api() -> Api {
    match Api::init() {
        Ok(api) => api,
        Err(e) => ipc::exit_with_error(Error::LibLoading(e.to_string())),
    }
}
