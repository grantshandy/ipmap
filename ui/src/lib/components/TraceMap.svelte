<script lang="ts">
  import GenericMap from "./GenericMap.svelte";

  import database from "tauri-plugin-ipgeo-api";
  import { type Hop } from "tauri-plugin-pcap-api";
  import { fade, fly } from "svelte/transition";
  import { type MapComponent } from "$lib/page.svelte";
  import GlobeSwitcher from "./GlobeSwitcher.svelte";
  import { renderLocationName } from "$lib/utils";

  type Props = {
    ip: string;
    hops: Hop[];
    close: () => void;
  };

  let { hops, close, ip }: Props = $props();
  let hopsOpen = $state(false);
  let map: MapComponent | undefined = $state();

  $effect(() => {
    if (!map) return;

    const locations = hops.filter((hop) => hop.loc != null);

    if (locations.length > 0) {
      const myLoc = locations[0].loc;
      if (myLoc == null) return; // for ts
      map.createMarker("1", myLoc.crd, 1);
    }

    for (let i = 1; i < locations.length; i++) {
      const from = locations[i - 1].loc;
      const to = locations[i].loc;

      if (!from || !to) continue; // shouldn't happen, for ts
      map.createArc(String(i), from.crd, to.crd, 0.75, "mixed");
    }

    const endpoint = locations[locations.length - 1].loc?.crd;
    if (endpoint) map.createMarker(String(locations.length), endpoint, 1);
  });
</script>

<GenericMap bind:map>
  <div class="absolute bottom-2 left-2 z-999">
    <GlobeSwitcher />
  </div>

  <div class="absolute top-2 right-2 z-999 flex items-center space-x-2">
    <button
      onclick={close}
      class="btn btn-sm join-item border-white text-xl select-none"
    >
      &#11148;
    </button>

    <h1
      class="bg-base-200 rounded-box border border-white px-2 py-1 text-xl font-semibold"
    >
      {ip}
    </h1>
  </div>

  {@render hopsPopup()}
</GenericMap>

{#snippet hopsPopup()}
  {#if !hopsOpen}
    <div
      class="absolute right-2 bottom-0 z-999 flex w-64 flex-col items-center select-none"
    >
      <button
        in:fade={{ duration: 100 }}
        onclick={() => (hopsOpen = true)}
        class="btn btn-xs rounded-b-none border-b-0 border-white"
      >
        View Hops
      </button>
    </div>
  {:else}
    <div
      transition:fly={{ y: 300, duration: 500 }}
      class="absolute right-2 bottom-0 z-999 flex w-64 flex-col select-none"
    >
      <button
        onclick={() => (hopsOpen = false)}
        class="btn btn-xs mx-auto translate-y-0.5 rounded-b-none border-b-0 border-white"
      >
        Close
      </button>
      <div
        class="bg-base-200 rounded-t-box flex h-64 w-full border-x border-t p-3"
      >
        <div class="grow space-y-1 overflow-y-scroll">
          {#each hops as hop, i}
            {@render renderHop(hop, i)}
          {/each}
        </div>
      </div>
    </div>
  {/if}
{/snippet}

{#snippet renderHop(hop: Hop, num: number)}
  {#if num != 0}
    <hr />
  {/if}

  <div>
    <h2 class="font-semibold">
      #{num + 1}:
      <span class="select-text">
        {hop.ips.length > 0 ? hop.ips.join(", ") : "Not Detected"}
      </span>
    </h2>

    {#if hop.loc}
      <button
        onclick={() => {
          if (!map || !hop.loc) return;
          map.flyToPoint(hop.loc.crd, 0.8);
        }}
        class="link text-left text-xs"
      >
        {renderLocationName(hop.loc.loc)}
      </button>
    {:else if hop.ips.length > 0}
      <p class="text-xs">Location not detected</p>
    {/if}
  </div>
{/snippet}

{#snippet lookupDns(ip: string)}
  <p>
    DNS:
    {#await database.lookupDns(ip)}
      Loading...
    {:then host}
      {#if host.status == "ok" && host.data != null}
        "{host.data}"
      {:else}
        Not Found
      {/if}
    {/await}
  </p>
{/snippet}
