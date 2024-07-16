<script lang="ts">
  import LocationName from "../components/LocationName.svelte";
  import MapView from "../components/MapView.svelte";
  import IpAddrInput from "../components/IpAddrInput.svelte";

  import { open } from "@tauri-apps/api/shell";
  import { Marker, marker, Map } from "leaflet";

  import { geoip, type Coordinate } from "../bindings";
  import { mkIcon } from "../map";

  let error: string | null = null;

  let map: Map;
  let selection: {
    ip: string;
    coord: Coordinate;
    marker: Marker;
  } | null = null;

  let searchTimeout: number;
  const validateAndSearch = async (ip: string) => {
    if (!map) return;
    clearTimeout(searchTimeout);

    const coord = await geoip.lookupIp(ip);
    if (!coord) {
      error = "IP Not Found in Database";
      setSearchIp(null);
      return;
    }

    error = null;
    searchTimeout = setTimeout(() => setSearchIp(ip, coord), 300);
  };

  const setSearchIp = (ip: string | null, coord?: Coordinate) => {
    if (selection) {
      selection.marker.remove();
      selection = null;
    }

    if (!coord || !map || !ip) return;

    selection = {
      ip,
      coord,
      marker: marker(coord, { icon: mkIcon(null, true) }).addTo(map),
    };

    if (map.getZoom() == 7) {
      map.panTo(coord);
    } else {
      map.flyTo(coord, 7);
    }
  };
</script>

<div class="flex grow space-x-2">
  <MapView bind:map>
    <div class="map-info-panel">
      <IpAddrInput bind:error onSearch={validateAndSearch} />
      {#if error}
        <p class="grow p-2 text-sm font-bold italic text-error">{error}</p>
      {/if}
      {#if selection}
        <h2 class="text-lg font-bold">IP Location Info</h2>
        {#await geoip.locationInfo(selection.coord) then info}
          {#if info}
            <p>Location: <LocationName {info} /></p>
          {/if}
        {/await}
        <button
          class="link text-sm italic"
          on:click={() =>
            open(
              `https://openstreetmap.org/#map=12/${selection?.coord.lat}/${selection?.coord.lng}`,
            )}
          >View in OpenStreetMap
        </button>
        <hr />
        {#await geoip.lookupDns(selection.ip) then dns}
          {#if dns}
            <p>Domain: <span class="code">{dns}</span></p>
          {/if}
        {/await}
        {#await geoip.lookupIpRange(selection.ip) then range}
          {#if range}
            <p>
              Block
              <span class="code break-words">{range.lower}</span>
              to
              <span class="code">{range.upper}</span>
            </p>
          {/if}
        {/await}
      {/if}
    </div>
  </MapView>
</div>
