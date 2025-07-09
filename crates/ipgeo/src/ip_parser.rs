use std::net::{Ipv4Addr, Ipv6Addr};

use compact_str::CompactString;

use crate::database::SteppedIp;

pub type IpParser<B> = fn(&[u8]) -> Option<SteppedIp<B>>;

pub mod str {
    use super::*;

    pub fn ipv4(record: &[u8]) -> Option<SteppedIp<Ipv4Addr>> {
        CompactString::from_utf8(record)
            .ok()
            .and_then(|s| s.parse::<Ipv4Addr>().ok())
            .map(SteppedIp)
    }

    pub fn ipv6(record: &[u8]) -> Option<SteppedIp<Ipv6Addr>> {
        CompactString::from_utf8(record)
            .ok()
            .and_then(|s| s.parse::<Ipv6Addr>().ok())
            .map(SteppedIp)
    }
}

pub mod num {
    use super::*;

    pub fn ipv4(record: &[u8]) -> Option<SteppedIp<Ipv4Addr>> {
        CompactString::from_utf8(record)
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .map(Ipv4Addr::from_bits)
            .map(SteppedIp)
    }

    pub fn ipv6(record: &[u8]) -> Option<SteppedIp<Ipv6Addr>> {
        CompactString::from_utf8(record)
            .ok()
            .and_then(|s| s.parse::<u128>().ok())
            .map(Ipv6Addr::from_bits)
            .map(SteppedIp)
    }
}
