use std::{
    io::{self, BufRead},
    net::IpAddr,
    process,
};

use child_ipc::{
    CaptureParams, Command, Error, PcapStatus, Response, TracerouteParams, TracerouteResponse,
};
use pcap_dyn::{Api, buf::CaptureTimeBuffer};

fn main() {
    let stdin = io::stdin();
    let mut line = String::new();

    if let Err(err) = stdin.lock().read_line(&mut line) {
        eprintln!("Must provide command on stdin.\n{err}");
        process::exit(1);
    }

    let cmd = match serde_json::from_str::<Command>(line.trim()) {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("Failed to parse command.\n{err}");
            process::exit(1);
        }
    };

    let resp: Result<Response, Error> = match cmd {
        Command::PcapStatus => get_pcap_status().map(Response::PcapStatus),
        Command::Capture(params) => run_capture(params),
        Command::TracerouteStatus => has_traceroute_privileges().map(Response::TracerouteStatus),
        Command::Traceroute(params) => run_traceroute(params).map(Response::TracerouteResponse),
    };

    send_response(resp);
}

fn get_pcap_status() -> Result<PcapStatus, Error> {
    let api = get_api();

    #[cfg(target_os = "linux")]
    if !caps::has_cap(None, caps::CapSet::Effective, caps::Capability::CAP_NET_RAW).unwrap_or(false)
    {
        return Err(Error::InsufficientPermissions);
    }

    let devices = api.devices().map_err(|e| Error::Runtime(e.to_string()))?;

    Ok(PcapStatus {
        devices,
        version: api.lib_version(),
    })
}

fn run_capture(params: CaptureParams) -> ! {
    let api = get_api();

    let cap = match api.open_capture(params.device) {
        Ok(capture) => capture,
        Err(e) => exit_with_error(Error::Runtime(e.to_string())),
    };
    let buf = CaptureTimeBuffer::start(cap, params.connection_timeout);

    loop {
        send_response(Ok(Response::CaptureSample(buf.connections())));
        std::thread::sleep(params.report_frequency);
    }
}

fn run_traceroute(params: TracerouteParams) -> Result<TracerouteResponse, Error> {
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
                    send_response(Ok(Response::TracerouteProgress(round)));
                }
            })
            .map_err(|e| Error::Runtime(e.to_string()))?;

        Ok(tracer.snapshot())
    })
    .map_err(|e| Error::Runtime(format!("trippy panicked: {e:?}")))??;

    let hops = snapshot
        .hops()
        .iter()
        .map(|h| h.addrs().copied().collect::<Vec<IpAddr>>())
        .collect::<Vec<Vec<IpAddr>>>();

    Ok(TracerouteResponse { hops })
}

fn get_api() -> Api {
    match Api::init() {
        Ok(api) => api,
        Err(e) => exit_with_error(Error::LibLoading(e.to_string())),
    }
}

fn has_traceroute_privileges() -> Result<bool, Error> {
    trippy_privilege::Privilege::discover()
        .map(|p| p.has_privileges())
        .map_err(|e| Error::Runtime(e.to_string()))
}

fn exit_with_error(error: Error) -> ! {
    send_response(Err(error));
    // nonzero would be correct, but harder to handle in the IPC layer.
    process::exit(0);
}

fn send_response(resp: Result<Response, Error>) {
    let s = serde_json::to_string(&resp).unwrap();
    println!("{s}");
}
