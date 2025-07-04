#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use child_ipc::{Command, Error, Response};

mod command;
mod ipc;

/// A small child application spawned by the functions in pcap_state::ipc.
/// It reads in a single child_ipc::Command on stdin at the start, then
/// returns (a series of) Result<child_ipc::Response, child_ipc::Error>, all in JSON.
fn main() {
    #[cfg(windows)]
    ipc::init();

    let response: Result<Response, Error> = match ipc::get_command() {
        Command::PcapStatus => command::get_pcap_status().map(Response::PcapStatus),
        Command::Capture(params) => command::run_capture(params),
        Command::TracerouteStatus => {
            command::has_net_raw_privileges().map(Response::TracerouteStatus)
        }
        Command::Traceroute(params) => {
            command::run_traceroute(params).map(Response::TracerouteResponse)
        }
    };

    ipc::send_response(response);
}
