<script lang="ts">
  import { geoip, type Coordinate } from "../bindings";
  import Link from "./Link.svelte";

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

    <Link
      className="select-none text-xs italic"
      href={`https://openstreetmap.org/#map=12/${coord.lat}/${coord.lng}`}
      >(View in OSM)
    </Link>
  </p>
{/await}
