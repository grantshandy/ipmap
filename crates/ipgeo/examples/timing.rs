use std::{env, path::PathBuf};

use ipgeo::{DatabaseTrait, GenericDatabase};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);

    let path = PathBuf::from(args.next().unwrap());

    let GenericDatabase::Ipv4(db) = ipgeo::detect(&path)? else {
        panic!("not ipv4");
    };

    println!("{:?}", db.get("1.1.1.1".parse().unwrap()));

    // std::thread::sleep(std::time::Duration::from_secs(40));

    // let before = Instant::now();
    // let samples = db.samples().collect::<Vec<_>>();

    // for ip in samples {
    //     let _ = db.get(ip);
    // }

    // println!("all samples took {}ms", before.elapsed().as_millis());

    Ok(())
}
