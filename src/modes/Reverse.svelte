<script lang="ts">
  import MapView from "../components/MapView.svelte";

  import { GeodesicLine } from "leaflet.geodesic";
  import { Map, marker, type LeafletMouseEvent, type Marker } from "leaflet";

  import { geoip, type Coordinate } from "../bindings";
  import { mkIcon } from "../map";
  import LocationInfoView from "../components/LocationInfoView.svelte";

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
    }, 5);
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
    <div class="map-info-panel overflow-y-auto">
      <h1
        class="select-none rounded-box border border-base-300 bg-base-100 px-3 py-2 font-semibold"
      >
        Nearest IP Location Blocks
      </h1>
      <div
        class="space-y-3 rounded-box border border-base-300 bg-base-100 px-3 py-2"
      >
        <LocationInfoView coord={result} />
      </div>
      <div
        class="overflow-y-auto rounded-box border border-base-300 bg-base-100 px-3 py-2"
      >
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
