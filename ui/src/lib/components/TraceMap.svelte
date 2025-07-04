<script lang="ts">
  import MapView from "./Map.svelte";

  import {
    database,
    regionNames,
    type Coordinate,
    type Hop,
  } from "$lib/bindings";
  import { geodesic, marker, type Map } from "leaflet";

  type Props = {
    myLocation: Coordinate;
    ip: string;
    hops: Hop[];
    close: () => void;
  };

  let { myLocation, hops, close, ip }: Props = $props();

  let map: Map | null = $state(null);

  const addGeodesicLine = (from: Coordinate, to: Coordinate) => {
    if (map)
      geodesic([from, to], {
        weight: 3,
        className: "trace",
      }).addTo(map);
  };

  $effect(() => {
    if (!map) return;

    marker(myLocation).addTo(map);

    const locations = hops.filter((hop) => hop.loc != null);

    if (locations.length > 0) {
      const firstLocation = locations[0].loc;
      if (firstLocation == null) return; // for ts

      addGeodesicLine(myLocation, firstLocation.crd);
    }

    for (let i = 1; i < locations.length; i++) {
      const from = locations[i - 1].loc;
      const to = locations[i].loc;

      if (!from || !to) continue; // shouldn't happen, for ts

      addGeodesicLine(from.crd, to.crd);
    }

    const endpoint = locations[locations.length - 1].loc?.crd;
    if (endpoint) marker(endpoint).addTo(map);
  });
</script>

<div class="flex h-full grow space-x-2">
  <MapView bind:map />
  <div class="h-full w-64 space-y-3 overflow-hidden p-2">
    <h1 class="text-xl font-bold underline">{ip}</h1>

    {#await database.lookupDns(ip) then host}
      {#if host.status == "ok" && host.data != null}
        <p>DNS: {host.data}</p>
      {/if}
    {/await}

    <button onclick={close} class="btn btn-sm">Back to Search</button>

    <h2 class="text-lg font-semibold">Hops:</h2>
    <div class="max-h-96 overflow-y-scroll">
      {#each hops as hop, i}
        {@render renderHop(hop, i + 1)}
        <hr />
      {/each}
    </div>
  </div>
</div>

{#snippet renderHop(hop: Hop, no: number)}
  <div class="py-1">
    <div class="flex items-center">
      <h3 class="grow text-xl font-semibold">#{no}:</h3>
      {#if hop.loc != null}
        <button
          class="btn btn-xs float-right"
          onclick={() => {
            if (hop.loc) map?.flyTo(hop.loc.crd, 7, { duration: 2 });
          }}
        >
          View
        </button>
      {:else if hop.ips.length == 0}
        <p class="text-sm italic">IP not detected</p>
      {/if}
    </div>
    {#if hop.ips.length > 0}
      {#if hop.ips.length == 1}
        <p>
          <span class="bg-base-300 bg-opacity-100 rounded-md p-0.5 font-mono">
            {hop.ips[0]}
          </span>
        </p>
      {:else}
        <ul class="ml-5 list-disc">
          {#each hop.ips as ip}
            <li>
              <span
                class="bg-base-300 bg-opacity-100 rounded-md p-0.5 font-mono"
              >
                {ip}
              </span>
            </li>
          {/each}
        </ul>
      {/if}

      {#if hop.loc != null}
        <p>Location:</p>
        <p class="text-sm">
          {`${hop.loc.loc.city ?? "Unknown City"}${hop.loc.loc.region ? `, ${hop.loc.loc.region}` : ""}`},
          {regionNames.of(hop.loc.loc.countryCode)}
        </p>
      {:else}
        <p>Location not detected</p>
      {/if}
    {/if}
  </div>
{/snippet}
