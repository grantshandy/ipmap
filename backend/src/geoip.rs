use std::{
    fs::File,
    io::Read,
    net::Ipv4Addr,
    ops::Deref,
    path::PathBuf,
    sync::{Arc, RwLock},
};

lazy_static::lazy_static! {
    static ref IPGEO_DB: crate::db_types::GeoDb = bincode::deserialize(&include_bytes!(concat!(env!("OUT_DIR"), "/encoded_db"))[..]).expect("failed to deserialize");
}

#[tauri::command]
pub async fn lookup_ip(ip: Ipv4Addr) -> Option<crate::db_types::Location> {
    tracing::info!("looking up {ip}");

    IPGEO_DB.get(&u32::from(ip)).cloned()
}
