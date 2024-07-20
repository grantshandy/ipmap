<script lang="ts">
  import { geoip } from "../bindings";

  export let ip: string;
</script>

<p class="overflow-x-auto text-xs">
  DNS:
  {#await geoip.lookupDns(ip)}
    <span class="loading loading-spinner loading-xs"></span>
  {:then dns}
    {#if dns}
      <span class="code">{dns}</span>
    {:else}
      <span>Unknown</span>
    {/if}
  {/await}
</p>

{#await geoip.lookupIpRange(ip) then range}
  {#if range}
    <p class="text-xs">
      Block
      <span class="code break-words">{range.lower}</span>
      to
      <span class="code">{range.upper}</span>
    </p>
  {/if}
{/await}
