use std::net::IpAddr;
use std::sync::RwLock;

use etherparse::{InternetSlice, SlicedPacket};
use ipgeolocate::{Locator, Service};
use once_cell::sync::Lazy;
use pcap::Device;

pub static IP_JSON_DOCUMENT: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new(String::new()));

pub async fn manage_ip(cap: Device) {
    let mut cap = cap.open().unwrap();

    let mut ip_index: Vec<IpAddr> = Vec::new();
    let mut lat_index: Vec<f64> = Vec::new();
    let mut lon_index: Vec<f64> = Vec::new();

    // I have to do this in a loop because cap.next() hangs or something on mac OSX and windows for some reason.
    // Not great for performance but it had to be done.
    loop {
        let packet = match cap.next() {
            Ok(data) => data,
            Err(_) => continue,
        };

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

                if !ip_rfc::global(&ip) {
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
                data.latitude.parse::<f64>().unwrap(),
                data.longitude.parse::<f64>().unwrap(),
                data.city,
            ),
            Err(_error) => match Locator::get_ipaddr(ip, Service::IpWhois).await {
                Ok(data) => (
                    data.latitude.parse::<f64>().unwrap(),
                    data.longitude.parse::<f64>().unwrap(),
                    data.city,
                ),
                Err(_error) => (0.0, 0.0, String::new()),
            },
        };

        if (!lat_index.contains(&lat) || !lon_index.contains(&lon)) && (lat != 0.0 && lon != 0.0) {
            lat_index.push(lat);
            lon_index.push(lon);

            // I do JSON here manually cuz we only need to serialize basic values.
            // We used to call serde_json on each new IP, that was a nightmare, this is much nicer :)
            IP_JSON_DOCUMENT.write().unwrap().push_str(&format!(
                r#"{{"ip":"{}","lat":"{}","lon":"{}","city":"{}",}},"#,
                ip, lat, lon, city
            ));

            println!("{} - ({}, {})", ip, lat, lon);
        }
    }
}

#[derive(Clone)]
pub struct IpAddress {
    pub ip: IpAddr,
    pub lat: f64,
    pub lon: f64,
    pub city: String,
}
