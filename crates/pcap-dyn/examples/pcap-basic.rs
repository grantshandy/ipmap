// use std::{collections::HashMap, net::IpAddr, thread, time::Duration};

// use pcap_dyn::ConnectionInfo;

fn main() {
    // let instance = &pcap_dyn::INSTANCE;
    // let api = instance.as_ref().expect("Failed to load pcap library");

    // let device = api
    //     .devices()
    //     .expect("couldn't get devices")
    //     .into_iter()
    //     .filter(|d| d.name == "wlp3s0")
    //     .next()
    //     .expect("no net device found");

    // println!("{device:#?}");

    // let cap = api
    //     .open_capture(device, Duration::from_secs(3), print_state)
    //     .unwrap();

    // thread::sleep(Duration::from_secs(90));
    // cap.stop();
    // thread::sleep(Duration::from_secs(1));
}

// fn print_state(state: HashMap<IpAddr, ConnectionInfo>) {
//     println!("\n............................");

//     for (ip, info) in state {
//         println!(
//             "{ip}: {} ({}/s) down | {} ({}/s) up",
//             human_bytes::human_bytes(info.down.total as f64),
//             human_bytes::human_bytes(info.down.avg_s),
//             human_bytes::human_bytes(info.up.total as f64),
//             human_bytes::human_bytes(info.up.avg_s),
//         );
//     }

//     println!("............................\n");
// }
