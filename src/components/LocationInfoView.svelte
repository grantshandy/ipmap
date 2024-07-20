<script lang="ts">
  import { geoip, type Coordinate } from "../bindings";
  import { open } from "@tauri-apps/plugin-shell";

  const countryNames = new Intl.DisplayNames("en", { type: "region" });

  export let coord: Coordinate;
</script>

{#await geoip.locationInfo(coord) then info}
  <p>
    {#if info}
      {#if info.city}
        {info.city},
      {/if}
      {#if info.state}
        {info.state},
      {/if}
      {countryNames.of(info.country_code)}
    {:else}
      No Location Data Found
    {/if}

    <button
      class="link text-xs italic select-none"
      on:click={() =>
        open(`https://openstreetmap.org/#map=12/${coord.lat}/${coord.lng}`)}
      >(View in OSM)
    </button>
  </p>
{/await}
