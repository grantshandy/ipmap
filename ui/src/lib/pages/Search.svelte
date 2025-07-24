<script lang="ts">
  import GenericMap from "$lib/components/GenericMap.svelte";
  import IpSearchBox from "$lib/components/IpSearchBox.svelte";

  import {
    database,
    type LookupInfo,
    type Location,
    type Result,
    renderLocationName,
  } from "$lib/bindings";
  import { fade } from "svelte/transition";
  import { type MapComponent } from "$lib/page.svelte";
  import GlobeSwitcher from "$lib/components/GlobeSwitcher.svelte";

  let map: MapComponent | undefined = $state();
  let result: { info: LookupInfo; ip: string } | string | null = $state(null);

  const search = async (input: Result<string, string> | null) => {
    if (!map) return;

    map.removeMarker("");

    if (input == null) {
      return;
    } else if (input.status == "error") {
      result = input.error;
      return;
    }

    const ip = input.data;
    const info = await database.lookupIp(ip);

    if (!info) {
      result = `"${ip}" not found in database`;
      return;
    }

    result = { info, ip };
    map.createMarker("", info.crd, 1);
    map.flyToPoint(info.crd, 0.8);
  };
</script>

<GenericMap bind:map>
  <div class=" absolute top-2 right-2 z-[999] flex items-center space-x-2">
    <GlobeSwitcher />
    <div class="rounded-box bg-base-300 border">
      <IpSearchBox {search} />
    </div>
  </div>

  {#if typeof result == "string"}
    {@render error(result)}
  {:else if result != null && typeof result == "object"}
    {@render info(result.ip, result.info.loc)}
  {/if}
</GenericMap>

{#snippet info(ip: string, loc: Location)}
  <div
    transition:fade={{ duration: 200 }}
    class="bg-base-200 rounded-box absolute right-2 bottom-2 z-[999] border p-2 text-right select-none"
  >
    <p class="underline">{ip}</p>
    <p class="text-sm">{renderLocationName(loc)}</p>
    {#await database.lookupDns(ip) then host}
      {#if host.status == "ok" && host.data != null}
        <p class="font-mono text-xs">DNS: {host.data}</p>
      {/if}
    {/await}
  </div>
{/snippet}

{#snippet error(msg: string)}
  <p
    transition:fade={{ duration: 200 }}
    class="rounded-box bg-error absolute bottom-2 left-2 z-[999] p-2 text-sm select-none"
  >
    {msg}
  </p>
{/snippet}
