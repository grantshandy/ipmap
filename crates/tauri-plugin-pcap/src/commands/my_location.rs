use std::{net::IpAddr, sync::OnceLock};

use ipgeo::{Coordinate, Location, LookupInfo};
use public_ip_address::response::LookupResponse;
use tauri::State;

static CACHED_LOCATION: OnceLock<Result<(IpAddr, Option<LookupInfo>), String>> = OnceLock::new();

/// Get a cached location response
pub async fn try_get_my_location() -> Result<(IpAddr, Option<LookupInfo>), String> {
    match CACHED_LOCATION.get() {
        Some(r) => r.clone(),
        None => {
            let l = perform_lookup().await;
            CACHED_LOCATION.get_or_init(|| l).clone()
        }
    }
}

/// Attempt to get the user's current [`LookupInfo`] from their IP address.
#[tauri::command]
#[specta::specta]
pub async fn my_location(
    state: State<'_, tauri_plugin_ipgeo::DbState>,
) -> Result<LookupInfo, String> {
    match try_get_my_location().await? {
        (_, Some(info)) => Ok(info),
        (ip, None) => match tauri_plugin_ipgeo::commands::lookup_ip(state, ip) {
            Some(info) => Ok(info),
            None => Err(format!("Your IP {ip} not found in loaded database")),
        },
    }
}

async fn perform_lookup() -> Result<(IpAddr, Option<LookupInfo>), String> {
    tracing::debug!("looking up public ip address with 'public-ip-address' crate");

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
