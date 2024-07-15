<script lang="ts">
  import MapView from "./MapView.svelte";
  import LocationName from "./LocationName.svelte";

  import { GeodesicLine } from "leaflet.geodesic";
  import { Map, marker, type LeafletMouseEvent, type Marker } from "leaflet";

  import { geoip, type Coordinate } from "../bindings";
  import { mkIcon } from "../map";

  const MAX_RANGES = 200;

  let map: Map;

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
    }, 20);
  }

  $: line.setLatLngs([query, result]);

  // add marker and line to map when created
  $: if (map) {
    map.invalidateSize();
    queryMarker.addTo(map);
    line.addTo(map);
  }
</script>

<div class="flex grow space-x-2">
  <MapView bind:map>
    <div
      class="absolute bottom-0 right-0 top-0 z-30 w-1/4 space-y-3 overflow-y-auto rounded-l-box bg-base-200/[0.8] p-2"
    >
      <h1 class="rounded-box bg-base-200 p-2 font-semibold">
        Nearest IP Location Blocks
      </h1>
      {#await geoip.locationInfo(result) then info}
        {#if info}
          <p class="rounded-box bg-base-200 p-2">
            <LocationName {info} />
          </p>
        {/if}
      {/await}
      <hr />
      <div class="overflow-y-auto rounded-box bg-base-200 p-2">
        {#await geoip.lookupIpBlocks(result) then ranges}
          {#if ranges.length > MAX_RANGES}
            <p class="text-sm italic">More ranges not shown...</p>
          {/if}
          <div class="grid grid-cols-2 overflow-y-auto text-xs">
            <span class="font-bold">From</span>
            <span class="font-bold">To</span>
            {#each ranges.slice(0, MAX_RANGES + 1) as range}
              <span>{range.lower}</span>
              <span>{range.upper}</span>
            {/each}
          </div>
        {/await}
      </div>
    </div>
  </MapView>
</div>
