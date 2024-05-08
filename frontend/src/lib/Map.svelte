<script>
    import InfoPane from "./InfoPane.svelte";

    import { listen } from "@tauri-apps/api/event";
    import { lookupIp } from "../bridge";

    import L from "leaflet";
    import "leaflet-providers";
    import "leaflet/dist/leaflet.css";

    let selected;

    let locs = {}; // { 'latitudelongitude': { loc, marker, ips: [] } }
    let map;

    listen("new_connection", (event) => {
        const ip = event.payload;

        lookupIp(ip).then((loc) => {
            const key = `${loc.latitude}${loc.longitude}`;

            if (map != null) {
                if (locs[key] != null) {
                    locs[key].ips.push(ip);
                } else {
                    locs[key] = {
                        loc,
                        marker: L.marker([loc.latitude, loc.longitude])
                            .on("click", () => (selected = locs[key]))
                            .addTo(map),
                        ips: [ip],
                    };
                }
            }
        });
    });

    const mapAction = (container) => {
        map = L.map(container, { preferCanvas: true }).setView([30, 0], 2);
        L.tileLayer.provider("OpenStreetMap.Mapnik").addTo(map);

        return {
            destroy: () => {
                map.remove();
                map = null;
            },
        };
    };

    const resizeMap = () => {
        if (map) {
            map.invalidateSize();
        }
    };
</script>

<svelte:window on:resize={resizeMap} />

<div class="w-full h-full flex space-x-2">
    <div class="grow select-none rounded-sm" use:mapAction></div>
    <InfoPane selection={selected} />
</div>
