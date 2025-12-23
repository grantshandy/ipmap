use std::{fs, net::Ipv6Addr};

use ipgeo::SingleDatabase;

#[tokio::main]
async fn main() {
    let db = SingleDatabase::<Ipv6Addr>::download(
        "https://github.com/sapics/ip-location-db/raw/refs/heads/main/dbip-city/dbip-city-ipv6-num.csv.gz",
        true,
        |v| println!("{:.2}", v * 100.0)
    ).await.unwrap();

    fs::write("single-db.bin", &postcard::to_allocvec(&db).unwrap()).unwrap()
}
