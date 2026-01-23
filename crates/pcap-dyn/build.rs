use std::{env, error, fs};

// These are the standard private ranges (RFC 1918) + Loopback + Multicast
const PRIVATE_RANGES: [&str; 6] = [
    "10.0.0.0/8",      // Private Class A
    "172.16.0.0/12",   // Private Class B
    "192.168.0.0/16",  // Private Class C
    "127.0.0.0/8",     // Loopback
    "224.0.0.0/4",     // Multicast
    "255.255.255.255", // Broadcast
];

fn main() -> Result<(), Box<dyn error::Error>> {
    fs::write(
        format!("{}/bpf_filter", env::var("OUT_DIR")?),
        build_bpf_filter(),
    )?;

    Ok(())
}

fn build_bpf_filter() -> Vec<u8> {
    let src_expr: String = PRIVATE_RANGES
        .iter()
        .map(|r| format!("src net {r}"))
        .collect::<Vec<_>>()
        .join(" or ");

    let dst_expr: String = PRIVATE_RANGES
        .iter()
        .map(|r| format!("dst net {r}"))
        .collect::<Vec<_>>()
        .join(" or ");

    // LOGIC:
    // 1. Must be IP or IPv6 (strip ARP, STP, etc)
    // 2. AND NOT (Src is Private AND Dst is Private)
    // 3. AND NOT (Broadcast or Multicast) - purely redundant safety
    let mut msg = format!(
        "(ip or ip6) and not ( ({src_expr}) and ({dst_expr}) ) and not (broadcast or multicast)",
    )
    .into_bytes();

    // null-terminator, expected by the C-library that consumes this
    msg.push(0);

    msg
}
