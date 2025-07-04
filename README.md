# Ipmap

```shell
 $ setcap cap_net_raw,cap_net_admin=eip target/release/pcap-child
```

## Contents
 - `/crates`
    - `desktop` - The main program entrypoint, starts tauri and generates Typescript IPC types.
    - `pcap-dyn` - Dynamic bindings to the [`libpcap`](https://www.tcpdump.org/) C library, modeled after the [`pcap`](https://crates.io/crates/pcap) crate.
    - `pcap-state` - UI state and methods for executing `pcap-child`.
    - `pcap-child` - A separate child process for executing privileged features such as packet capture and traceroute.
    - `child-ipc` - Shared types between `pcap-state` and `pcap-child` for IPC.
    - `ipgeo` - Data structures for representing ip-geolocation databases.
    - `ipgeo-state` - UI state and methods for loading and switching ip-geolocation databases.
 - `/ui` - The desktop UI, written with Svelte and Typescript.
