use std::net::Ipv4Addr;

pub mod database {
    include!(concat!(env!("OUT_DIR"), "/database.rs"));
}

#[tauri::command]
pub async fn lookup_ip(ip: Ipv4Addr) -> Option<database::db_types::Location> {
    tracing::info!("looking up {ip}");

    if database::DATABASE.is_none() {
        tracing::info!("no built in database");
    }

    database::DATABASE
        .as_ref()
        .map(|db| db.get(&u32::from(ip)).cloned())
        .flatten()
}
