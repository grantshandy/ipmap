use std::{ops::Deref, sync::RwLock};
use std::net::IpAddr;

use once_cell::sync::Lazy;
use serde::Serialize;
use etherparse::{SlicedPacket, InternetSlice};
use ipgeolocate::{Locator, Service};
use pcap::{Capture, Device};

pub static IP_INDEX: Lazy<RwLock<Vec<IpAddress>>> = Lazy::new(|| RwLock::new(Vec::new()));
pub static IP_JSON_DOCUMENT: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new(String::new()));

pub async fn manage_ip() {
    #[cfg(unix)]
    let mut cap = Capture::from_device(Device::lookup().unwrap()).unwrap().open().unwrap();
    #[cfg(windows)]
    let mut cap = user_select_device();

    let mut ip_index: Vec<IpAddr> = Vec::new();
    let mut lat_index: Vec<f64> = Vec::new();
    let mut lon_index: Vec<f64> = Vec::new();

    while let Ok(packet) = cap.next() {
        match SlicedPacket::from_ethernet(&packet) {
            Err(value) => println!("error {:?}", value),
            Ok(value) => {
                let ip = match match value.ip {
                    Some(data) => data,
                    None => continue,
                } {
                    InternetSlice::Ipv4(ip) => IpAddr::V4(ip.source_addr()),
                    InternetSlice::Ipv6(ip, _) => IpAddr::V6(ip.source_addr()),
                };

                if !ip.is_global() {
                    continue;
                }

                try_add_ip(ip, &mut ip_index, &mut lat_index, &mut lon_index).await;
            }
        }
    }
}

async fn try_add_ip(ip: IpAddr, ip_index: &mut Vec<IpAddr>, lat_index: &mut Vec<f64>, lon_index: &mut Vec<f64>) {
    if !ip_index.contains(&ip) {
        ip_index.push(ip);

        let (lat, lon, city) = match Locator::get_ipaddr(ip, Service::IpApi).await {
            Ok(data) => (data.latitude.parse::<f64>().unwrap(), data.longitude.parse::<f64>().unwrap(), data.city),
            Err(_error) => (0.0, 0.0, String::new()),
        };

        if !lat_index.contains(&lat) || !lon_index.contains(&lon) {
            lat_index.push(lat);
            lon_index.push(lon);

            IP_INDEX.write().unwrap().push(IpAddress { ip, lat, lon, city });
            create_json_document();

            println!("{} - ({}, {})", ip, lat, lon);
        }
    }
}

fn create_json_document() {
    let json = serde_json::to_string(IP_INDEX.read().unwrap().deref()).unwrap();

    IP_JSON_DOCUMENT.write().unwrap().clear();
    IP_JSON_DOCUMENT.write().unwrap().push_str(&json);
}

#[cfg(windows)]
fn user_select_device() -> Device {
    let mut devices = Device::list().unwrap();
    if devices.is_empty() {
        println!("Found no device to listen on, maybe you need to run as an Adminstrator");
        std::process::exit(1);
    }
    println!("Select which device to listen on: (choose the number of the item)");
    for (i, d) in devices.iter().enumerate() {
        println!("{}: {:?}", i, d);
    }
    use std::io;

    let mut input = String::new();
    let n = loop {
        io::stdin().read_line(&mut input).unwrap();
        match input.trim().parse() {
            Ok(n) => {
                if n < devices.len() {
                    break n;
                } else {
                    println!("Invalid choice, try again");
                    input.clear();
                }
            }
            Err(_) => {
                println!("Invalid choice, try again");
                input.clear();
            }
        }
    };
    println!("Listening on {:?}", devices[n]);
    devices.remove(n)
}

#[derive(Serialize)]
pub struct IpAddress {
    pub ip: IpAddr,
    pub lat: f64,
    pub lon: f64,
    pub city: String,
}
