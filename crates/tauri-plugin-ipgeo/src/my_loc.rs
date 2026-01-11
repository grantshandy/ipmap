use std::{net::IpAddr, sync::OnceLock};

use ipgeo::{Coordinate, Location, LookupInfo};
use public_ip_address::response::LookupResponse;

static CACHED_LOCATION: OnceLock<Result<(IpAddr, Option<LookupInfo>), String>> = OnceLock::new();

/// Get a cached location response
pub async fn get() -> Result<(IpAddr, Option<LookupInfo>), String> {
    match CACHED_LOCATION.get() {
        Some(r) => r.clone(),
        None => {
            let l = perform_lookup().await;
            CACHED_LOCATION.get_or_init(|| l).clone()
        }
    }
}

async fn perform_lookup() -> Result<(IpAddr, Option<LookupInfo>), String> {
    tracing::info!("looking up public ip address");

    match public_ip_address::perform_lookup(None)
        .await
        .map_err(|e| e.to_string())?
    {
        LookupResponse {
            ip,
            latitude: Some(lat),
            longitude: Some(lng),
            country_code: Some(country_code),
            city,
            region,
            ..
        } => Ok((
            ip,
            Some(LookupInfo {
                crd: Coordinate {
                    lat: lat as f32,
                    lng: lng as f32,
                },
                loc: Location {
                    city,
                    region,
                    country_code,
                },
            }),
        )),
        LookupResponse { ip, .. } => Ok((ip, None)),
    }
}
