<script lang="ts">
    import { pcap, database } from "../bindings";
    import MapView from "./Map.svelte";
    import { type Map, marker, type Marker } from "leaflet";

    let map: Map | null = $state(null);

    const markers: Record<string, Marker> = {};

    pcap.onConnStart(async (ip, info) => {
        if (!map) return;

        const loc = await database.lookupIp(ip);

        if (!loc) {
            // TODO: handle case where location is not found
            return;
        }

        markers[ip] = marker([loc.crd.lat, loc.crd.lng])
            .bindPopup(ip)
            .addTo(map);
    });

    pcap.onConnEnd((ip) => {
        markers[ip]?.remove();
        delete markers[ip];
    });
</script>

<div class="w-[700px] h-[700px] flex">
    <MapView bind:map />
</div>
