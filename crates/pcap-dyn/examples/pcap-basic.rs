use std::{thread, time::Duration};

use pcap_dyn::Api;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let api: &'static Api = pcap_dyn::LIBRARY.as_ref()?;

    let devices = api.devices()?;

    println!("{devices:#?}");

    let device = devices
        .into_iter()
        .find(|d| d.name == "wlp3s0")
        .expect("no net device found");

    println!("{device:?}");

    let mut cap = api.open_capture(device)?;

    let recv = cap.start();

    thread::spawn(move || {
        for _ in recv {
            // println!("{p:?}");
        }

        println!("stopped capturing");
    });

    thread::sleep(Duration::from_secs(10));
    cap.stop();
    thread::sleep(Duration::from_secs(1));

    Ok(())
}
