const COMMANDS: &[&str] = &[
    "start_capture",
    "stop_capture",
    "init_pcap",
    "traceroute_enabled",
    "run_traceroute",
    "print_error",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
