use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub ip: String,
    pub latitude: f64,
    pub longitude: f64,
    pub city: String,
    pub country: String,
    pub org: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GeolocateError {
    pub message: String,
}

pub async fn geolocate(ip: IpAddr) -> Result<Location, GeolocateError> {
    match get_ip_api(&ip).await {
        Ok(location) => Ok(location),
        Err(_) => get_ip_api_co(&ip).await,
    }
}

async fn get_ip_api_co(ip: &IpAddr) -> Result<Location, GeolocateError> {
    let response = match reqwest::get(format!("https://ipapi.co/{}/json/", ip.to_string())).await {
        Ok(response) => response,
        Err(error) => {
            return Err(GeolocateError {
                message: error.to_string(),
            })
        }
    };

    let text = match response.text().await {
        Ok(text) => text,
        Err(error) => {
            return Err(GeolocateError {
                message: error.to_string(),
            })
        }
    };

    let json: Value = match serde_json::from_str(&text) {
        Ok(json) => json,
        Err(error) => {
            return Err(GeolocateError {
                message: error.to_string(),
            })
        }
    };

    let latitude: f64 = match json.get("latitude") {
        Some(lat) => match lat.as_f64() {
            Some(lat) => lat,
            None => {
                return Err(GeolocateError {
                    message: "latitude returned from ip-api not a number".to_string(),
                })
            }
        },
        None => {
            return Err(GeolocateError {
                message: "no latitude returned from ip-api".to_string(),
            })
        }
    };

    let longitude: f64 = match json.get("longitude") {
        Some(lon) => match lon.as_f64() {
            Some(lon) => lon,
            None => {
                return Err(GeolocateError {
                    message: "longitude returned from ip-api not a number".to_string(),
                })
            }
        },
        None => {
            return Err(GeolocateError {
                message: "no longitude returned from ip-api".to_string(),
            })
        }
    };

    let city: String = match json.get("city") {
        Some(city) => city.to_string().replace('"', ""),
        None => {
            return Err(GeolocateError {
                message: "no city returned from ip-api".to_string(),
            })
        }
    };

    let country: String = match json.get("country_name") {
        Some(country) => country.to_string().replace('"', ""),
        None => {
            return Err(GeolocateError {
                message: "no country returned from ip-api".to_string(),
            });
        }
    };

    let org: Option<String> = json.get("org").map(|x| x.to_string().replace('"', ""));

    let ip = ip.to_string();

    Ok(Location {
        ip,
        latitude,
        longitude,
        city,
        country,
        org,
    })
}

async fn get_ip_api(ip: &IpAddr) -> Result<Location, GeolocateError> {
    let response = match reqwest::get(format!(
        "http://ip-api.com/json/{}?fields=lat,lon,city,country,org",
        ip.to_string()
    ))
    .await
    {
        Ok(response) => response,
        Err(error) => {
            return Err(GeolocateError {
                message: error.to_string(),
            })
        }
    };

    let text = match response.text().await {
        Ok(text) => text,
        Err(error) => {
            return Err(GeolocateError {
                message: error.to_string(),
            })
        }
    };

    let json: Value = match serde_json::from_str(&text) {
        Ok(json) => json,
        Err(error) => {
            return Err(GeolocateError {
                message: error.to_string(),
            })
        }
    };

    let latitude: f64 = match json.get("lat") {
        Some(lat) => match lat.as_f64() {
            Some(lat) => lat,
            None => {
                return Err(GeolocateError {
                    message: "latitude returned from ip-api not a number".to_string(),
                })
            }
        },
        None => {
            return Err(GeolocateError {
                message: "no latitude returned from ip-api".to_string(),
            })
        }
    };

    let longitude: f64 = match json.get("lon") {
        Some(lon) => match lon.as_f64() {
            Some(lon) => lon,
            None => {
                return Err(GeolocateError {
                    message: "longitude returned from ip-api not a number".to_string(),
                })
            }
        },
        None => {
            return Err(GeolocateError {
                message: "no longitude returned from ip-api".to_string(),
            })
        }
    };

    let city: String = match json.get("city") {
        Some(city) => city.to_string().replace('"', ""),
        None => {
            return Err(GeolocateError {
                message: "no city returned from ip-api".to_string(),
            })
        }
    };

    let country: String = match json.get("country") {
        Some(country) => country.to_string().replace('"', ""),
        None => {
            return Err(GeolocateError {
                message: "no country returned from ip-api".to_string(),
            });
        }
    };

    let org: Option<String> = json.get("org").map(|x| x.to_string().replace('"', ""));

    let ip = ip.to_string();

    Ok(Location {
        ip,
        latitude,
        longitude,
        city,
        country,
        org,
    })
}
