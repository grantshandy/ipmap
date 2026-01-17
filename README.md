# Ipmap

> [!NOTE]
> Old versions of ipmap are now located at [`grantshandy/ipmap-old`](https://github.com/grantshandy/ipmap-old).
>
> This version of ipmap is not complete, but is nearly done. Release tarballs will be available soon.

A GUI viewer for [`saphics/ip-location-db`](https://github.com/sapics/ip-location-db?tab=readme-ov-file#city) ip-geolocation databases,
which can display your computer's live network traffic and perform trace routes.

| Search                              | Capture                               | Traceroute                                  |
|-------------------------------------|---------------------------------------|---------------------------------------------|
| ![search](./screenshots/search.png) | ![capture](./screenshots/capture.png) | ![traceroute](./screenshots/traceroute.png) |

## Requirements
On Linux, [WebKitGTK](https://repology.org/project/webkitgtk/versions) is the only uncommon dependency.

### Network Monitor
 - On Linux, install [`libpcap`](https://repology.org/project/libpcap/versions).
 - On Windows, install [Npcap](https://npcap.org) with network capture for non-administrator users.
 - On macOS, `libpcap` is already installed.

### Traceroute
On Windows, you must enable a firewall rule to send ICMP packets for the traceroute feature.

```powershell
New-NetFirewallRule -DisplayName "ICMPv4 Ipmap Allow" -Name ICMPv4_IPMAP_ALLOW -Protocol ICMPv4 -Action Allow
New-NetFirewallRule -DisplayName "ICMPv6 Ipmap Allow" -Name ICMPv6_IPMAP_ALLOW -Protocol ICMPv6 -Action Allow

Enable-NetFirewallRule ICMPv4_IPMAP_ALLOW
Enable-NetFirewallRule ICMPv6_IPMAP_ALLOW
```

## Building
Requirements:
 - [Tauri prerequisites](https://tauri.app/start/prerequisites/)
 - [Rust](https://rust-lang.org)
 - [`tauri-cli`](https://v2.tauri.app/reference/cli/)
 - [`pnpm`](https://pnpm.io/)

```sh
 $ pnpm install
 $ pnpm -r build
 $ cargo build --release --package ipmap-child
 $ cargo tauri build
```

## Source Contents
 - `/crates`
    - `/desktop` - The main program entrypoint.
    - `/ipgeo` - Data structures for representing ip-geolocation databases.
    - `/tauri-plugin-ipgeo` - Svelte store library and tauri plugin for loading/switching/querying ip-geolocation databases.
    - `/pcap-dyn` - Dynamic bindings to the [`libpcap`](https://www.tcpdump.org/) C library, modeled after the [`pcap`](https://crates.io/crates/pcap) crate.
    - `/tauri-plugin-pcap` - Svelte store library and tauri plugin for capturing on network devices and running traceroutes.
    - `/child` - A separate child process (`ipmap-child`) for executing privileged features such as packet capture and traceroute.
    - `/child-ipc` - Shared types between `tauri-plugin-pcap` and `ipmap-child` for IPC methods executed on `ipmap-child`.
 - `/ui` - The desktop UI, written with Svelte and Typescript.

## TODO:
 - [ ] Animated demo in readme.
 - [ ] Check macOS compatibility.
 - [ ] Light/dark mode with system. Try to match native UI?
 - [ ] Translate user interface.
 - [ ] Get default network interfaces with native APIs.
 - [ ] Add multi-lingual readmes (zh, es).
 - [ ] Remove `public-ip-address` bloated dependency, create our own solution.
 - [ ] Add settings dialog.
   - [ ] Different map layers?
   - [ ] Capture report frequency and connection timeout.
 - [ ] Reverse location-to-ip-block search
 - [ ] Set child permissions through UI pkexec
 - [ ] NixOS packaging
 - [ ] Fix old archlinux AUR script.
