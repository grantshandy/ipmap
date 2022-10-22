use std::net::IpAddr;

pub async fn geolocate(ip: IpAddr) -> String {
    let response = match reqwest::get(format!("http://ip-api.com/json/{}", ip.to_string())).await {
        Ok(response) => response,
        Err(error) => return format!("{{\"error\":\"{error}\"}}"),
    };

    match response.text().await {
        Ok(text) => text,
        Err(error) => return format!("{{\"error\":\"{error}\"}}"),
    }
}
