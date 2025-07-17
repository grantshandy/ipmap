// use std::{collections::HashMap, net::IpAddr, thread, time::Duration};

// use pcap_dyn::ConnectionInfo;

use std::{thread, time::Duration};

use pcap_dyn::Api;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Api::init()?;

    let devices = api.devices()?;
    let device = devices
        .into_iter()
        .filter(|d| d.name == "wlo1")
        .next()
        .expect("no net device found");

    println!("{device:?}");

    let cap = api.open_capture(device)?;

    let recv = cap.start();

    thread::spawn(move || {
        for p in recv {
            println!("{p:?}");
        }

        println!("stopped capturing");
    });

    thread::sleep(Duration::from_secs(10));
    cap.stop();
    thread::sleep(Duration::from_secs(1));

    Ok(())
}
