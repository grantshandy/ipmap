use std::{net::IpAddr, sync::OnceLock};

use ipgeo::{Coordinate, Location};
use public_ip_address::response::LookupResponse;

use crate::LookupInfo;

static CACHED_LOCATION: OnceLock<Result<MyLocationResponse, String>> = OnceLock::new();

#[derive(Clone)]
pub enum MyLocationResponse {
    Full(LookupInfo),
    JustIp(IpAddr),
}

/// Get a cached location response
pub async fn get() -> Result<MyLocationResponse, String> {
    match CACHED_LOCATION.get() {
        Some(r) => r.clone(),
        None => {
            let l = perform_lookup().await;
            CACHED_LOCATION.get_or_init(|| l).clone()
        }
    }
}

async fn perform_lookup() -> Result<MyLocationResponse, String> {
    tracing::info!("looking up public ip address");

    let res = public_ip_address::perform_lookup(None)
        .await
        .map_err(|e| e.to_string())?;

    let me = if let LookupResponse {
        latitude: Some(lat),
        longitude: Some(lng),
        country_code: Some(country_code),
        city,
        region,
        ..
    } = res
    {
        MyLocationResponse::Full(LookupInfo {
            crd: Coordinate {
                lat: lat as f32,
                lng: lng as f32,
            },
            loc: Location {
                city,
                region,
                country_code,
            },
        })
    } else {
        MyLocationResponse::JustIp(res.ip)
    };

    Ok(me)
}
