use std::{thread, time::Duration};

use child_ipc::{Command, TracerouteParams};
use pcap_state::ipc;

fn main() {
    println!(
        "{:#?}",
        ipc::call_child_process(Command::TracerouteStatus, false)
    );
    println!(
        "{:#?}",
        ipc::call_child_process(Command::TracerouteStatus, true)
    );

    let (items, exit) = ipc::spawn_child_iterator(
        Command::Traceroute(TracerouteParams {
            ip: "1.1.1.1".parse().unwrap(),
            max_rounds: 5,
        }),
        true,
    )
    .expect("Failed to start capture");

    thread::spawn(move || {
        for item in items {
            println!("{item:?}");
        }
        println!("pipe closed");
    });

    thread::sleep(Duration::from_secs(15));
    exit().expect("Failed to stop capture");
    println!("stopped capture");
    thread::sleep(Duration::from_secs(5));
}
