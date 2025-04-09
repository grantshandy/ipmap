fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pcap = pcap_dyn::Pcap::init()?;

    let devices = pcap.get_devices()?;

    let device = devices
        .iter()
        .filter(|d| d.name == "enp2s0")
        .next()
        .ok_or("No device found")?;

    println!("Using device: {device:?}");

    let cap = pcap.open(device)?;
    let recv = cap.start();

    while let Ok(packet) = recv.recv() {
        println!("{:?}", packet);
    }

    // also exists cap.stop()

    Ok(())
}
