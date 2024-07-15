# ipmap
A map that displays connected IP addresses

![demo](./resources/demo.gif)

Variables for building:
 - `IPV4NUM_DB`: The path to the built-in ipv4-num.csv database (optional)
 - `IPV4NUM_DB_ATTRIBUTION`: An attribution message for that database's copyright (required if `IPV6NUM_DB` set)
 - `IPV6NUM_DB`: The path to the built-in ipv6-num.csv database (optional)
 - `IPV6NUM_DB_ATTRIBUTION`: An attribution message for that database's copyright (required if `IPV6NUM_DB` set)

(Example)
```shell
 $ IPV4NUM_DB="/path/to/dbip-city-ipv4-num.csv"
 $ IPV4NUM_DB_ATTRIBUTION="IP Geolocation by DB-IP"
```

# Dependencies
 - [Rust](https://rust-lang.org)
 - [NodeJS](https://nodejs.org)
 - [`cargo-tauri`](https://crates.io/crates/tauri-cli)

 - MacOS: `libpcap`
 - Windows [npcap](https://npcap.org)
 - Linux: Webkit2GTK, `libpcap`

# Building
```shell
 $ cargo tauri build
```

## TODO:
 - [X] geodesic arc lines to connections
 - [X] animate geodesic lines
 - [X] differentiate outgoing and incoming lines
 - [x] fix arc animation memory leak bug
 - [x] ipv6 support
 - [x] unload database
 - [ ] visual traceroute
 - [x] novel data structure for maintaining a live-updated list of
  current connections based on the stream of packets. Also determines incoming/outgoing/mixed status.
 - [x] ^ maintain arc animations from this state
 - [x] dark/light mode
 - [x] fix capture -> search marker update bug
 - [x] reverse search? move a marker on the map and it shows the geographically closest blocks.
 - [ ] detect undersea cables in traceroute
 - [ ] identify internet access points in traceroute
 - [ ] load `.csv.gz` and `.csv.7z` compressed databases
 