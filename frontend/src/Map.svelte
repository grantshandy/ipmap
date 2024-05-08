<script>
    import { listen } from "@tauri-apps/api/event";
    import { lookupIp } from "./bridge";

    import L from "leaflet";
    import "leaflet/dist/leaflet.css";

    let ips = [];
    let locs = {}; // { 'latitudelongitude': { loc, marker, ips: [] } }
    let map;

    listen("new_connection", (event) => {
        const ip = event.payload;

        lookupIp(ip).then((loc) => {
            const key = `${loc.latitude}${loc.longitude}`;

            if (locs[key] != null) {
                locs[key].ips.push(ip);
                locs[key].marker.bindPopup(locs[key].ips.join("<br>")).update();
            } else {
                locs[key] = {
                    loc,
                    marker: L.marker([loc.latitude, loc.longitude])
                        .bindPopup(`${ip}`)
                        .addTo(map),
                    ips,
                };
            }
        });
    });

    const mapAction = (container) => {
        map = L.map(container, { preferCanvas: true }).setView([30, 0], 2);
        L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
            attribution: `&copy;<a href="https://www.openstreetmap.org/copyright" target="_blank">OpenStreetMap</a>`,
            maxZoom: 9,
        }).addTo(map);

        return {
            destroy: () => {
                map.remove();
                map = null;
            },
        };
    };
</script>

<svelte:window
    on:resize={() => {
        if (map) {
            map.invalidateSize();
        }
    }}
/>
<div class="w-full h-full rounded-xl" use:mapAction></div>
