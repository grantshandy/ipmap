# ipmap
A map that displays connected IP addresses

![demo](./resources/arc-demo.gif)

 - https://github.com/sapics/ip-location-db
 - https://npcap.com/

Variables for building:
 - `IPV4NUM_DB`: The path to the built-in ipv4-num.csv database (optional)
 - `IPV4NUM_DB_ATTRIBUTION`: The copyright message for that database (required if `IPV4NUM_DB` set)

(Example)
```shell
$ IPV4NUM_DB="/path/to/dbip-city-ipv4-num.csv"
$ IPV4NUM_DB_ATTRIBUTION="IP Geolocation by DB-IP"
```

## TODO:
 - [X] geodesic arc lines to connections
 - [X] animate geodesic lines
 - [X] differentiate outgoing and incoming lines
 - [x] fix arc animation memory leak bug
 - [ ] ipv6 support
 - [x] unload database
 - [ ] visual traceroute
 - [x] novel data structure for maintaining a live-updated list of
  current connections based on the stream of packets. Also determines incoming/outgoing/mixed status.
 - [x] ^ maintain arc animations from this state
 - [ ] dark/light mode