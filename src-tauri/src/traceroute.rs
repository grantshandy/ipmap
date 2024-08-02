use std::net::IpAddr;

use ipdb_city::{Coordinate, DatabaseQuery};
use serde::{Deserialize, Serialize};
use tauri::State;
use trippy_core::Builder;
use trippy_privilege::Privilege;
use ts_rs::TS;

use crate::{
    geoip::{lookup_ip, my_location},
    GlobalDatabases, PUBLIC_IP,
};

#[tauri::command]
pub async fn traceroute(
    loaded_databases: State<'_, GlobalDatabases>,
    ip: IpAddr,
    options: TracerouteOptions,
    database: DatabaseQuery,
) -> Result<Vec<Hop>, String> {
    if !ip_rfc::global(&ip) {
        return Err(format!("{ip} not global"));
    }

    let tracer = Builder::new(ip)
        .max_ttl(options.max_ttl)
        .max_flows(1)
        .max_rounds(Some(options.max_rounds))
        .build()
        .map_err(|e| format!("failed to init tracer: {e}"))?;

    tracer
        .run()
        .map_err(|e| format!("failed to run tracer: {e}"))?;

    let my_location = my_location(loaded_databases.clone(), database.clone()).await?;

    let snapshot = tracer.snapshot();
    let mut flow = Vec::new();

    flow.push(Hop {
        ip: Some(*PUBLIC_IP),
        coord: Some(my_location),
    });

    for hop in snapshot.hops() {
        if let Some(ip) = hop.addrs().nth(0) {
            if !ip_rfc::global(&ip) {
                continue;
            }

            flow.push(Hop {
                ip: Some(*ip),
                coord: lookup_ip(loaded_databases.clone(), database.clone(), *ip)
                    .await
                    .ok()
                    .flatten(),
            });
        } else {
            flow.push(Hop {
                ip: None,
                coord: None,
            });
        }
    }

    if !flow.last().is_some_and(|hop| hop.ip == Some(ip)) {
        flow.push(Hop {
            ip: Some(ip),
            coord: lookup_ip(loaded_databases.clone(), database.clone(), ip)
                .await
                .ok()
                .flatten(),
        });
    }

    Ok(flow)
}

#[tauri::command]
pub async fn is_privileged() -> bool {
    Privilege::acquire_privileges()
        .ok()
        .is_some_and(|p| p.has_privileges())
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct TracerouteOptions {
    max_rounds: usize,
    max_ttl: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Hop {
    ip: Option<IpAddr>,
    coord: Option<Coordinate>,
}
