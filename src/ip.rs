use std::net::IpAddr;
use std::{ops::Deref, sync::RwLock};

use etherparse::{InternetSlice, SlicedPacket};
use ipgeolocate::{Locator, Service};
use once_cell::sync::Lazy;
use pcap::{Active, Capture};
use serde::Serialize;

pub static IP_INDEX: Lazy<RwLock<Vec<IpAddress>>> = Lazy::new(|| RwLock::new(Vec::new()));
pub static IP_JSON_DOCUMENT: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new(String::new()));

pub async fn manage_ip(cap: Capture<Active>) {
    let mut cap = cap;

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

async fn try_add_ip(
    ip: IpAddr,
    ip_index: &mut Vec<IpAddr>,
    lat_index: &mut Vec<f64>,
    lon_index: &mut Vec<f64>,
) {
    if !ip_index.contains(&ip) {
        ip_index.push(ip);

        let (lat, lon, city) = match Locator::get_ipaddr(ip, Service::IpApi).await {
            Ok(data) => (
                data.latitude.parse::<f64>().expect("My bad from rust."),
                data.longitude.parse::<f64>().expect("My bad from rust."),
                data.city,
            ),
            Err(_error) => (0.0, 0.0, String::new()),
        };

        if !lat_index.contains(&lat) || !lon_index.contains(&lon) {
            lat_index.push(lat);
            lon_index.push(lon);

            IP_INDEX
                .write()
                .expect("My bad from rust.")
                .push(IpAddress { ip, lat, lon, city });
            create_json_document();

            println!("{} - ({}, {})", ip, lat, lon);
        }
    }
}

fn create_json_document() {
    let json = serde_json::to_string(IP_INDEX.read().expect("My bad from rust.").deref()).expect("My bad from rust.");

    IP_JSON_DOCUMENT.write().expect("My bad from rust.").clear();
    IP_JSON_DOCUMENT.write().expect("My bad from rust.").push_str(&json);
}

#[derive(Serialize)]
pub struct IpAddress {
    pub ip: IpAddr,
    pub lat: f64,
    pub lon: f64,
    pub city: String,
}
