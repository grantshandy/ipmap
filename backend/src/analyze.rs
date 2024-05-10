use std::net::IpAddr;

#[tauri::command]
pub async fn dns_lookup_addr(ip: String) -> Result<Option<String>, String> {
    tracing::info!("looking up dns for {ip}");

    ip.parse::<IpAddr>()
        .map(|ip| dns_lookup::lookup_addr(&ip).ok())
        .map_err(|e| e.to_string())
}
