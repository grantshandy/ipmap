<script lang="ts">
  import { geoip } from "../bindings";
  import { type CaptureLocation } from "../map";
  import LocationName from "./LocationName.svelte";

  export let selection: CaptureLocation;
</script>

<h2 class="text-lg font-semibold">IP Location Information</h2>

{#await geoip.locationInfo(selection.coord) then info}
  {#if info}
    <p><LocationName {info} /></p>
  {/if}
{/await}

<hr />

<h3>Connections:</h3>

<ul class="space-y-1 divide-y">
  {#each selection.ips as ip}
    <li>
      <p>{ip}</p>
      {#await geoip.lookupDns(ip) then dns}
        {#if dns}
          <p class="code overflow-x-auto text-xs">
            {dns}
          </p>
        {/if}
      {/await}
    </li>
  {/each}
</ul>
