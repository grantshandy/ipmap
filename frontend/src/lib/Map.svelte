<script lang="ts">
    import InfoPane from "./InfoPane.svelte";

    import { listen } from "@tauri-apps/api/event";
    import { lookupIp } from "../utils";
    import type { LocationSelection } from "../utils";

    import { marker, map, tileLayer, Map } from "leaflet";
    import "leaflet-providers";
    import "leaflet/dist/leaflet.css";

    let selected: LocationSelection | null = null;

    let locs: { [id: string]: LocationSelection } = {};
    let mapInstance: Map | null = null;

    listen("new_connection", (event) => {
        const ip = event.payload as string;

        lookupIp(ip).then((loc) => {
            if (mapInstance != null && loc != null) {
                const key = `${loc.latitude}${loc.longitude}`;

                if (locs[key] != null) {
                    locs[key].ips.push(ip);
                } else {
                    locs[key] = {
                        loc,
                        marker: marker([loc.latitude, loc.longitude])
                            .on("click", () => (selected = locs[key]))
                            .addTo(mapInstance),
                        ips: [ip],
                    };
                }
            }
        });
    });

    const mapAction = (container: HTMLDivElement) => {
        mapInstance = map(container, { preferCanvas: true }).setView(
            [30, 0],
            2,
        );
        tileLayer.provider("OpenStreetMap.Mapnik").addTo(mapInstance);

        return {
            destroy: () => {
                if (mapInstance) {
                    mapInstance.remove();
                    mapInstance = null;
                }
            },
        };
    };

    const resizeMap = () => {
        if (mapInstance) {
            mapInstance.invalidateSize();
        }
    };
</script>

<svelte:window on:resize={resizeMap} />

<div class="w-full h-full flex space-x-2">
    <div class="grow select-none rounded-sm" use:mapAction></div>
    <InfoPane selection={selected} />
</div>

<!-- <style>
    @import "leaflet/dist/leaflet.css";
</style> -->
