use std::{collections::HashMap, net::IpAddr, process, sync::Arc};

use etherparse::{InternetSlice, SlicedPacket};
use log::{error, info};
use pcap::Device;
use tokio::sync::watch::Sender;

use crate::geolocate::geolocate;

pub async fn capture(stream_tx: Sender<String>, device: Device) {
    let mut cap = match device.open() {
        Ok(cap) => cap,
        Err(error) => {
            error!("Error opening device: {error}");
            process::exit(1);
        }
    };

    let mut registry: HashMap<IpAddr, String> = HashMap::new();
    let stream_tx = Arc::new(stream_tx);

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

        if let Some(location) = registry.get(&ip) {
            send_to_client(stream_tx.clone(), location.to_owned());
        } else {
            info!("Making request for {ip}");

            match geolocate(ip.clone()).await {
                Ok(location) => {
                    let json_string = serde_json::to_string(&location).unwrap();

                    registry.insert(ip, json_string.clone());
                    send_to_client(stream_tx.clone(), json_string);
                }
                Err(error) => {
                    error!("Error geolocating: {}", error.message);
                }
            };
        }
    }
}

fn send_to_client(stream_tx: Arc<Sender<String>>, location: String) {
    if let Err(error) = stream_tx.send(location) {
        error!("Error to send across channel from capture: {error}");
    };
}
