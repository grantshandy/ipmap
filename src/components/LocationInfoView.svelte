<script lang="ts">
  import { geoip, type Coordinate } from "../bindings";
  import { open } from "@tauri-apps/plugin-shell";

  const countryNames = new Intl.DisplayNames("en", { type: "region" });

  export let coord: Coordinate;
</script>

<div class="space-y-3 rounded-box bg-base-200 px-3 py-2">
  <h2 class="grow text-lg font-semibold">IP Location Information</h2>

  {#await geoip.locationInfo(coord) then info}
    {#if info}
      <p>
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
      open(`https://openstreetmap.org/#map=12/${coord.lat}/${coord.lng}`)}
    >View in OpenStreetMap
  </button>
</div>
