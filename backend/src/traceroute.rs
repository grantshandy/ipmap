use std::net::IpAddr;

use trippy_core::{Builder, State};
use trippy_privilege::Privilege;

#[tauri::command]
pub async fn traceroute(ip: IpAddr) -> Result<Vec<IpAddr>, String> {
    if !ip_rfc::global(&ip) {
        return Err(format!("{ip} not global"));
    }

    let tracer = Builder::new(ip)
        .max_ttl(64)
        .max_flows(1)
        .max_rounds(Some(2))
        .build()
        .map_err(|e| format!("init tracer: {e}"))?;

    tracing::info!("running traceroute");
    tracer.run().map_err(|e| format!("run tracer: {e}"))?;
    tracing::info!("done.");

    let mut ips = Vec::new();

    // todo: how to flatten iterator?
    for hop in tracer.snapshot().hops(State::default_flow_id()) {
        ips.extend(hop.addrs().filter(|ip| ip_rfc::global(ip)));
    }

    Ok(ips)
}

#[tauri::command]
pub async fn is_privileged() -> bool {
    Privilege::acquire_privileges()
        .ok()
        .is_some_and(|p| p.has_privileges())
}