use std::path::PathBuf;

use ipgeo::{Ipv4Database, Ipv6Database};

pub type DiskDatabases = (Vec<(PathBuf, Ipv4Database)>, Vec<(PathBuf, Ipv6Database)>);
