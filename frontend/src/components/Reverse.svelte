<script lang="ts">
  import MapView from "./MapView.svelte";
  import { GeodesicLine } from "leaflet.geodesic";
  import { createMap, mkIcon, type MapStore } from "../stores";
  import { marker, type LeafletMouseEvent, type Marker } from "leaflet";
  import { geoip, type Coordinate } from "../bindings";

  let map: MapStore;

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
  $: if ($map) {
    $map.inst.invalidateSize();
    queryMarker.addTo($map.markerLayer);
    line.addTo($map.arcLayer);
  }
</script>

<!-- TODO: Fix Weird Overflow -->
<div class="flex grow space-x-2 overflow-y-auto">
  <div class="flex h-full grow">
    <MapView bind:map />
  </div>
  <div class="h-full w-1/4 select-none space-y-2">
    <h1 class="rounded-box bg-base-200 p-2 font-semibold">
      Nearest IP Location Blocks
    </h1>
    {#await geoip.locationInfo(result) then info}
      {#if info}
        <p class="rounded-box bg-base-200 p-2">
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
    <div class="rounded-box bg-base-200 overflow-y-auto p-2">
      <div class="grid grid-cols-2 overflow-y-auto text-xs">
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
