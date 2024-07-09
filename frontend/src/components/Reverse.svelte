<script lang="ts">
    import MapView from "./MapView.svelte";
    import LocationInfo from "./LocationInfo.svelte";

    import { map, mkIcon } from "../stores/map";
    import { onMount } from "svelte";
    import {
        marker,
        type LeafletEvent,
        type LeafletMouseEvent,
        type Marker,
    } from "leaflet";
    import { nearestLocation, type Location } from "../bindings";
    import { database } from "../stores/database";
    import { GeodesicLine } from "leaflet.geodesic";

    const countryNames = new Intl.DisplayNames("en", { type: "region" });

    let searchTimeout: number;
    const onMarkerMove = (ev: LeafletEvent) => {
        const event = ev as LeafletMouseEvent;

        clearTimeout(searchTimeout);
        setTimeout(async () => {
            result = await nearestLocation(
                event.latlng.lat,
                event.latlng.lng,
                $database,
            );
        }, 25);
    };

    let queryMarker: Marker = marker([0, 0], {
        draggable: true,
        autoPan: true,
        icon: mkIcon(null),
    }).on("move", onMarkerMove);

    let result: Location | null = null;
    const line = new GeodesicLine([queryMarker.getLatLng(), [10, 10]]);
    $: if (result)
        line.setLatLngs([
            queryMarker.getLatLng(),
            [result.latitude, result.longitude],
        ]);

    onMount(() => {
        if (!$map) return;

        line.addTo($map.arcLayer);
        queryMarker.addTo($map.markerLayer);
    });
</script>

<div class="grow flex space-x-2">
    <MapView />
    <div class="w-1/4 space-y-2 bg-base-200 p-2 rounded-box">
        {#if result}
            <h1>Nearest IP Location Block</h1>
            <p>
                Location:

                {#if result.city}
                    {result.city},
                {/if}
                {#if result.state}
                    {result.state},
                {/if}
                {countryNames.of(result.country_code)}
            </p>
        {/if}
    </div>
</div>
