<script lang="ts">
  import { open } from "@tauri-apps/api/shell";
  import { geoip } from "../bindings";
  import { map, type IpLocation } from "../stores/map";
  import MapView from "./MapView.svelte";

  const countryNames = new Intl.DisplayNames("en", { type: "region" });

  let query = "";
  let error: string | null = null;

  $: validateAndSearch(query, true);
  $: if (!error || error) setTimeout(() => map.invalidateSize(), 10);

  let searchTimeout: number;
  const validateAndSearch = async (ip: string, pause: boolean) => {
    if (pause) clearTimeout(searchTimeout);

    if (ip.length == 0) {
      error = null;
      map.setSearchIp(null);
      map.resetView();
      return;
    }

    if (!(await geoip.validateIp(ip))) {
      error = "Invalid Address";
      map.setSearchIp(null);
      return;
    }

    if (!(await geoip.lookupIp(ip))) {
      error = "IP Not Found in Database";
      map.setSearchIp(null);
      return;
    }

    error = null;
    if (pause) {
      searchTimeout = setTimeout(() => map.setSearchIp(ip), 300);
    } else {
      map.setSearchIp(ip);
    }
  };

  let selection: IpLocation | null | undefined = null;
  $: selection = $map?.selection;
</script>

<div class="flex grow space-x-2">
  <MapView />
  <div class="w-1/4 space-y-2 rounded-box bg-base-200 p-2">
    <input
      class="input input-sm input-bordered w-full grow"
      class:border-error={error}
      placeholder="IPv4 Address"
      bind:value={query}
    />
    {#if error}
      <p class="grow p-2 text-sm font-bold italic text-error">{error}</p>
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
            `https://openstreetmap.org/#map=12/${selection.coord.lat}/${selection.coord.lng}`,
          )}
        >View in OpenStreetMap
      </button>
      <hr />
      {#each selection.ips as ip}
        <h3 class="font-semibold">{ip}:</h3>
        {#await geoip.lookupDns(ip) then dns}
          {#if dns}
            <p>Domain: <span class="code">{dns}</span></p>
          {/if}
        {/await}
        {#await geoip.lookupIpRange(ip) then range}
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
      {/each}
    {/if}
  </div>
</div>
