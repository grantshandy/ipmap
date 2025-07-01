use std::time::Duration;

use isolate_parent::commands;

fn main() {
    let status = commands::pcap_status().unwrap();

    println!("{status:#?}");

    let device = status
        .devices
        .iter()
        .find(|d| d.name == "wlp3s0")
        .unwrap()
        .clone();

    let cap = commands::start_capture_child(isolate_ipc::CaptureParams {
        device,
        connection_timeout: Duration::from_secs(3),
        report_frequency: Duration::from_secs(2),
    })
    .unwrap();

    for connections in cap {
        println!("{connections:?}");
    }

    // let (tx, rx) = std::sync::mpsc::channel();

    // let params = isolate_ipc::TracerouteParams {
    //     ip: "1.1.1.1".parse().unwrap(),
    //     max_rounds: 10,
    // };

    // std::thread::spawn(move || {
    //     while let Ok(round) = rx.recv() {
    //         println!("Round: {round}");
    //     }
    // });

    // let resp = isolate_parent::run_traceroute(params, tx).unwrap();

    // for hop in resp.hops {
    //     println!("Hop: {:?}", hop);
    // }
}
