use pcap::Device;
use pcap::Capture;

fn main() {
    let mut cap = Capture::from_device("wlp1s0").unwrap().open().unwrap();

    while let Ok(packet) = cap.next() {
        println!("received packet! {:?}", packet);
    }

    // for device in pcap::Device::list().unwrap() {
    //     println!("Found device! {:?}", device);

    //     // now you can create a Capture with this Device if you want.
    //     // see example/easylisten.rs for how
    // }
}