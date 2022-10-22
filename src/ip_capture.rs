use std::{
    net::IpAddr,
    process,
    sync::{Arc, Mutex},
};

use etherparse::{InternetSlice, SlicedPacket};
use log::error;
use pcap::Device;
use tokio::sync::watch::Sender;

use crate::geolocate;

pub async fn capture(sender: Sender<String>, device: Device) {
    let mut cap = match device.open() {
        Ok(cap) => cap,
        Err(error) => {
            error!("Error opening device: {error}");
            process::exit(1);
        }
    };

    let sender = Arc::new(sender);

    loop {
        let packet = match cap.next_packet() {
            Ok(packet) => packet,
            Err(error) => {
                error!("Error getting packet: {}", error);
                continue;
            }
        };

        let parsed_packet = match SlicedPacket::from_ethernet(&packet) {
            Ok(packet) => packet,
            Err(error) => {
                error!("Error parsing packet: {error}");
                continue;
            }
        };

        let ip: IpAddr = match parsed_packet.ip {
            Some(slice) => match slice {
                InternetSlice::Ipv4(ip, _) => IpAddr::V4(ip.source_addr()),
                InternetSlice::Ipv6(ip, _) => IpAddr::V6(ip.source_addr()),
            },
            None => continue,
        };

        if !ip_rfc::global(&ip) {
            continue;
        }

        let sender = sender.clone();

        tokio::spawn(async move {
            // let json = geolocate::geolocate(ip).await;
            
            if let Err(error) = sender.send(ip.to_string()) {
                error!("Error to send across channel from capture: {error}");
            };
        });
    }
}
