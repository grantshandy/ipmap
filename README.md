# ipmap

A map that displays connected IP addresses

![demo](./resources/demo.gif)

Variables for building:

- `IPGEO4_DB`: The path to the built-in ipv4-num.csv database (optional)
- `IPGEO4_DB_ATTR`: An attribution message for that database's copyright (required if `IPGEO4_DB` set)
- `IPGEO6_DB`: The path to the built-in ipv6-num.csv database (optional)
- `IPGEO6_DB_ATTR`: An attribution message for that database's copyright (required if `IPGEO6_DB` set)

- `LIB` (windows): The path to `npcap-sdk\Lib\x64`.

(Example)

```shell
 $ IPV4NUM_DB="/path/to/dbip-city-ipv4-num.csv"
 $ IPV4NUM_DB_ATTRIBUTION="IP Geolocation by DB-IP"
```

# Dependencies

- [Rust](https://rust-lang.org)
- [NodeJS](https://nodejs.org)

- Windows [npcap](https://npcap.org)
- Linux: Webkit2GTK, `libpcap`

# Building

```shell
 $ npx tauri build
```
