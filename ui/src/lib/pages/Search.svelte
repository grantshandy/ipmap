<script lang="ts">
  import GenericMap from "$lib/components/GenericMap.svelte";
  import IpAddrInput from "$lib/components/IpAddrInput.svelte";

  import { type LookupInfo, type Result } from "tauri-plugin-ipgeo-api";

  import { renderLocationName } from "$lib/utils";

  import database from "tauri-plugin-ipgeo-api";

  import { fade } from "svelte/transition";
  import { type MapComponent } from "$lib/page.svelte";

  type LookupState = {
    info: LookupInfo;
    ip: string;
  };

  let query: string | null = $state(null);
  let map: MapComponent | undefined = $state();
  let result: LookupState | string | null = $state(null);

  const search = async () => {
    if (!map) return;
    map.removeMarker("");

    if (!query) return;
    const info = await database.lookupIp(query);

    if (info) {
      result = { info, ip: query };
      map.createMarker("", info.crd, 1);
      map.flyToPoint(info.crd, 0.8);
    } else {
      result = `"${query}" not found in database`;
    }
  };
</script>

<GenericMap bind:map {searchbox} {infobox} />

{#snippet searchbox()}
  <IpAddrInput class="join-item" bind:value={query} onchange={search} />

  <button
    class="btn btn-sm btn-primary join-item"
    disabled={query == null}
    onclick={search}
  >
    Search
  </button>
{/snippet}

{#snippet infobox()}
  {#if typeof result == "string"}
    {@render errorbox(result)}
  {:else if result != null && typeof result == "object"}
    {@render locationinfo(result)}
  {/if}
{/snippet}

{#snippet errorbox(msg: string)}
  <p
    transition:fade={{ duration: 200 }}
    class="rounded-box bg-error p-2 text-sm select-none"
  >
    {msg}
  </p>
{/snippet}

{#snippet locationinfo(result: LookupState)}
  <div
    transition:fade={{ duration: 200 }}
    class="bg-base-200 rounded-box min-w-64 border p-2 text-right select-none"
  >
    <p class="text-2xl underline">{result.ip}</p>
    <p class="text-sm">{renderLocationName(result.info.loc)}</p>
    {#await database.lookupDns(result.ip) then host}
      {#if host.status == "ok" && host.data != null}
        <p class="font-mono text-xs">DNS: {host.data}</p>
      {/if}
    {/await}

    <!-- TODO: add "view traceroute" button -->
  </div>
{/snippet}
