<script lang="ts">
    import MapView from "./MapView.svelte";
    import { GeodesicLine } from "leaflet.geodesic";

    import { map, mkIcon, database } from "../stores";
    import {
        LatLng,
        marker,
        type LeafletMouseEvent,
        type Marker,
    } from "leaflet";
    import { nearestLocation, type LocationBlock } from "../bindings";

    const countryNames = new Intl.DisplayNames("en", { type: "region" });

    let queryLoc: LatLng = new LatLng(0, 0);
    let result: LocationBlock | null = null;

    let queryMarker: Marker = marker([0, 0], {
        draggable: true,
        autoPan: true,
        icon: mkIcon(null),
    })
        .setLatLng([0, 0])
        .on("move", (ev) => (queryLoc = (ev as LeafletMouseEvent).latlng));
    const line = new GeodesicLine([queryMarker.getLatLng(), [0, 0]], {
        className: "map-line",
    });

    // update result from queryLoc
    $: nearestLocation(queryLoc.lat, queryLoc.lng, $database).then(
        (loc) => (result = loc),
    );

    // update line from result location
    $: if (result)
        line.setLatLngs([
            queryMarker.getLatLng(),
            [result.location.latitude, result.location.longitude],
        ]);

    // add marker and line to map when created
    $: if ($map)
        (async () => {
            $map.inst.invalidateSize();
            queryMarker.addTo($map.markerLayer);
            line.addTo($map.arcLayer);
        })();
</script>

<!-- TODO: Fix Weird Overflow -->
<div class="grow flex space-x-2 overflow-y-auto">
    <div class="flex grow h-full">
        <MapView />
    </div>
    <div
        class="w-1/4 space-y-2 select-none h-full"
    >
        <h1 class="bg-base-200 rounded-box p-2 font-semibold">
            Nearest IP Location Blocks
        </h1>
        {#if result}
            <p class="bg-base-200 rounded-box p-2">
                {#if result.location.city}
                    {result.location.city},
                {/if}
                {#if result.location.state}
                    {result.location.state},
                {/if}
                {countryNames.of(result.location.country_code)}
            </p>
            <hr />
            <div class="bg-base-200 p-2 rounded-box overflow-y-auto">
                <div class="grid grid-cols-2 text-xs overflow-y-auto">
                    <span class="font-bold">From</span>
                    <span class="font-bold">To</span>
                    {#each result.blocks as range}
                        <span>{range.lower}</span>
                        <span>{range.upper}</span>
                    {/each}
                </div>
            </div>
        {/if}
    </div>
</div>
