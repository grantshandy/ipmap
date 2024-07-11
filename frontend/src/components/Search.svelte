<script lang="ts">
  import { open } from "@tauri-apps/api/shell";
  import { geoip, type Coordinate } from "../bindings";
  import { mkIcon, type IpLocation, type MapStore } from "../stores/map";
  import MapView from "./MapView.svelte";
  import { Marker, marker } from "leaflet";

  let map: MapStore;

  const countryNames = new Intl.DisplayNames("en", { type: "region" });

  let query = "";
  let error: string | null = null;

  $: validateAndSearch(query, true);
  $: if (!error || error) setTimeout(() => map.invalidateSize(), 10);

  let searchTimeout: number;
  const validateAndSearch = async (ip: string, pause: boolean) => {
    if (!map) return;
    clearTimeout(searchTimeout);

    if (ip.length == 0) {
      error = null;
      setSearchIp(null);
      map.resetView();
      return;
    }

    if (!(await geoip.validateIp(ip))) {
      error = "Invalid Address";
      setSearchIp(null);
      return;
    }

    let coord: Coordinate | null;
    if (!(coord = await geoip.lookupIp(ip))) {
      error = "IP Not Found in Database";
      setSearchIp(null);
      return;
    }

    error = null;
    searchTimeout = setTimeout(() => setSearchIp(ip, coord), 300);
  };

  let selection: { ip: string; coord: Coordinate; marker: Marker } | null =
    null;
  const setSearchIp = async (ip: string | null, coord?: Coordinate) => {
    if (selection) {
      selection.marker.remove();
      selection = null;
    }

    if (!coord || !$map || !ip) return;

    selection = {
      ip,
      coord,
      marker: marker(coord, { icon: mkIcon(null, true) }).addTo($map.inst),
    };

    $map.inst.flyTo(coord, 7);
  };
</script>

<div class="flex grow space-x-2">
  <MapView bind:map />
  <div class="rounded-box bg-base-200 w-1/4 space-y-2 p-2">
    <input
      class="input input-sm input-bordered w-full grow"
      class:border-error={error}
      placeholder="IPv4 Address"
      bind:value={query}
    />
    {#if error}
      <p class="text-error grow p-2 text-sm font-bold italic">{error}</p>
    {/if}
    {#if selection}
      <h2 class="text-lg font-bold">IP Location Info</h2>
      {#await geoip.locationInfo(selection.coord) then info}
        {#if info}
          <p>
            Location:

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
            <span class="code break-words">
              {range.lower}
            </span>
            to
            <span class="code">
              {range.upper}
            </span>
          </p>
        {/if}
      {/await}
    {/if}
  </div>
</div>
