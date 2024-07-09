<script lang="ts">
    import MapView from "./MapView.svelte";

    import { map, mkIcon, database } from "../stores";
    import {
        marker,
        type LeafletEvent,
        type LeafletMouseEvent,
        type Marker,
    } from "leaflet";
    import { myLocation, nearestLocation, type Location } from "../bindings";
    import { GeodesicLine } from "leaflet.geodesic";

    const countryNames = new Intl.DisplayNames("en", { type: "region" });

    let result: Location | null = null;

    // updates the result from the location of the query marker.
    let searchTimeout: number;
    const onMarkerMove = (ev: LeafletEvent) => {
        clearTimeout(searchTimeout);
        setTimeout(async () => {
            const event = ev as LeafletMouseEvent;

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

    // a geodesic line, reactively linked to the closest block
    const line = new GeodesicLine([queryMarker.getLatLng(), [10, 10]], {
        className: "map-line",
    });
    $: if (result)
        line.setLatLngs([
            queryMarker.getLatLng(),
            [result.latitude, result.longitude],
        ]);

    // add marker and line to map when created
    $: if ($map)
        (async () => {
            $map.inst.invalidateSize();

            const loc = await myLocation($database);
            queryMarker
                .setLatLng([loc.latitude, loc.longitude])
                .addTo($map.markerLayer);
            line.addTo($map.arcLayer);
        })();
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
