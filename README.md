# ipmap
A map that displays connected IP addresses.

![screenshot](screenshot.png)

## Dependencies
ipmap requires `libpcap` and `webkit2gtk` on Linux.

## Using
Download from the releases page or build it yourself, then give it permissions for `pcap`, or run as root.

Once the window opens, it should take a few seconds for the pins to start appearing on the map, you can zoom in to try to provoke it into adding some from the openstreetmap servers.

You can get the name of the city and the ip of a marker by either right or left clicking on the marker (depends on your operating system).

- <kbd>f</kbd> - Toggle fullscreen
- <kbd>q</kbd> - Quit the window
- <kbd>c</kbd> - View credits

## Testing
If you really want to see it go crazy just download a torrent for a few seconds. The ip geolocation API service will rate limit you to 100 new markers per minute though.

## Building
```
$ cargo build --release
```