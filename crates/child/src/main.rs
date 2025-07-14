#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use child_ipc::{ChildError, Command, Response};

mod command;
mod ipc;

fn main() {
    // initialize the named pipe between child and parent processes.
    #[cfg(windows)]
    ipc::windows::init();

    let response: Result<Response, ChildError> = match ipc::get_command() {
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
