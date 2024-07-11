<script lang="ts">
    import MapView from "./MapView.svelte";
    import { GeodesicLine } from "leaflet.geodesic";
    import { map, mkIcon } from "../stores";
    import { marker, type LeafletMouseEvent, type Marker } from "leaflet";
    import { geoip, type Coordinate } from "../bindings";

    const countryNames = new Intl.DisplayNames("en", { type: "region" });

    let query: Coordinate = { lat: 0, lng: 0 };
    let result: Coordinate = query;

    const queryMarker: Marker = marker([0, 0], {
        draggable: true,
        autoPan: true,
        icon: mkIcon(null),
    }).on("move", (ev) => (query = (ev as LeafletMouseEvent).latlng));
    const line = new GeodesicLine([queryMarker.getLatLng(), [0, 0]], {
        className: "map-line",
    });

    // update result from query after 10 ms dragging pause
    let timeout: number;
    $: if (query) {
        clearTimeout(timeout);
        timeout = setTimeout(async () => {
            result = await geoip.nearestLocation(query);
        }, 10); 
    }

    $: line.setLatLngs([query, result]);

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
    <div class="w-1/4 space-y-2 select-none h-full">
        <h1 class="bg-base-200 rounded-box p-2 font-semibold">
            Nearest IP Location Blocks
        </h1>
        {#await geoip.locationInfo(result) then info}
            {#if info}
                <p class="bg-base-200 rounded-box p-2">
                    {#if info.city}
                        {info.city},
                    {/if}
                    {#if info.state}
                        {info.state},
                    {/if}
                    {countryNames.of(info.country_code)}
                </p>
            {/if}
        {/await}
        <hr />
        <div class="bg-base-200 p-2 rounded-box overflow-y-auto">
            <div class="grid grid-cols-2 text-xs overflow-y-auto">
                <span class="font-bold">From</span>
                <span class="font-bold">To</span>
                {#await geoip.lookupIpBlocks(result) then ranges}
                    {#each ranges as range}
                        <span>{range.lower}</span>
                        <span>{range.upper}</span>
                    {/each}
                {/await}
            </div>
        </div>
    </div>
</div>
