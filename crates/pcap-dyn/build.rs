use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        format!("{}/bpf_filter", env::var("OUT_DIR")?),
        build_bpf_filter(),
    )?;

    Ok(())
}

fn build_bpf_filter() -> String {
    // These are the standard private ranges (RFC 1918) + Loopback + Multicast
    let private_ranges = [
        "10.0.0.0/8",      // Private Class A
        "172.16.0.0/12",   // Private Class B
        "192.168.0.0/16",  // Private Class C
        "127.0.0.0/8",     // Loopback
        "224.0.0.0/4",     // Multicast
        "255.255.255.255", // Broadcast
    ];

    // We build a string like: "(net 10.0.0.0/8 or net 192.168.0.0/16 ...)"
    // Note: In BPF, 'net' checks both src and dst if not specified.
    // But to be precise for "Src is Private AND Dst is Private", we need explicit checks.

    let clause_src: Vec<String> = private_ranges
        .iter()
        .map(|r| format!("src net {}", r))
        .collect();

    let clause_dst: Vec<String> = private_ranges
        .iter()
        .map(|r| format!("dst net {}", r))
        .collect();

    let src_expr = clause_src.join(" or ");
    let dst_expr = clause_dst.join(" or ");

    // LOGIC:
    // 1. Must be IP or IPv6 (strip ARP, STP, etc)
    // 2. AND NOT (Src is Private AND Dst is Private)
    // 3. AND NOT (Broadcast or Multicast) - purely redundant safety
    format!(
        "(ip or ip6) and not ( ({}) and ({}) ) and not (broadcast or multicast)",
        src_expr, dst_expr
    )
}
