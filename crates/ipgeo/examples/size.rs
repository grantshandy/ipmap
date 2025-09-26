use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    os::unix::fs::MetadataExt,
    path::PathBuf,
    time::Instant,
};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use ipgeo::{CombinedDatabase, IpLookupTable, Ipv4Database, Ipv6Database, location::LocationStore};
use serde::{Serialize, de::DeserializeOwned};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);

    let v4 = PathBuf::from(args.next().unwrap());
    let v6 = PathBuf::from(args.next().unwrap());

    read_combined(&v4, &v6);

    print_info(
        "ipv4 only",
        file_size(&v4),
        Ipv4Database::from_csv(GzDecoder::new(File::open(v4).unwrap()), true).unwrap(),
    );

    print_info(
        "ipv6 only",
        file_size(&v6),
        Ipv6Database::from_csv(GzDecoder::new(File::open(v6).unwrap()), true).unwrap(),
    );

    Ok(())
}

fn read_combined(v4_path: &PathBuf, v6_path: &PathBuf) {
    let mut ipv4 = IpLookupTable::new();
    let mut ipv6 = IpLookupTable::new();
    let mut locations = LocationStore::default();

    ipgeo::reader::csv::read(
        GzDecoder::new(File::open(v4_path).unwrap()),
        true,
        &mut ipv4,
        &mut locations,
    )
    .unwrap();
    ipgeo::reader::csv::read(
        GzDecoder::new(File::open(v6_path).unwrap()),
        true,
        &mut ipv6,
        &mut locations,
    )
    .unwrap();

    print_info(
        "combined",
        file_size(v4_path) + file_size(v6_path),
        CombinedDatabase::new(ipv4, ipv6, locations),
    );
}

fn print_info<T: Serialize + DeserializeOwned>(name: &str, orig_size: u64, item: T) {
    println!("---------- {name} ----------");

    println!(
        "original file: {}",
        human_bytes::human_bytes(orig_size as f64)
    );

    print_serialize("postcard", postcard::to_allocvec(&item).unwrap(), |bytes| {
        let _: T = postcard::from_bytes(&bytes).unwrap();
    });

    print_serialize(
        "bincode",
        bincode::serde::encode_to_vec(&item, bincode::config::standard()).unwrap(),
        |bytes| {
            bincode::serde::decode_from_slice::<T, _>(&bytes, bincode::config::standard()).unwrap();
        },
    );
}

fn print_serialize(method: &str, bytes: Vec<u8>, deser_cb: fn(&[u8])) {
    println!("{method}: {}", human_bytes::human_bytes(bytes.len() as f64));
    print_deserialize_time(&format!("{method} uncompressed"), deser_cb, &bytes);

    let mut compressor = GzEncoder::new(Vec::with_capacity(bytes.len()), Compression::best());
    compressor.write_all(&bytes).unwrap();

    let compressed = compressor.finish().unwrap();
    println!(
        "{method} gzip compressed: {} ({:.2}%)",
        human_bytes::human_bytes(compressed.len() as f64),
        (compressed.len() as f64 / bytes.len() as f64) * 100.0
    );

    print_deserialize_time(
        &format!("{method} gzip compressed"),
        |compressed_bytes| {
            let mut buf = Vec::with_capacity(compressed_bytes.len());
            let mut decoder = GzDecoder::new(compressed_bytes);
            decoder.read_to_end(&mut buf).unwrap();
            deser_cb(&buf)
        },
        &compressed,
    );
}

fn print_deserialize_time(method: &str, cb: impl Fn(&[u8]), bytes: &[u8]) {
    let start = Instant::now();
    cb(bytes);
    println!(
        "{method} deserializing took {}ms",
        start.elapsed().as_millis()
    );
}

fn file_size(path: &PathBuf) -> u64 {
    fs::metadata(path).unwrap().size()
}
