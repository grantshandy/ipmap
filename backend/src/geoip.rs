use std::net::Ipv4Addr;

pub mod database {
    include!(concat!(env!("OUT_DIR"), "/database.rs"));
}

#[tauri::command]
pub async fn lookup_ip(ip: Ipv4Addr) -> Option<database::Location> {
    database::DATABASE
        .as_ref()
        .map(|db| db.get(&u32::from(ip)).cloned())
        .flatten()
}

#[tauri::command]
pub async fn load_internal_database() {
    lazy_static::initialize(&database::DATABASE);

    if database::DATABASE.is_none() {
        tracing::warn!("no internal database set");
    }
}
