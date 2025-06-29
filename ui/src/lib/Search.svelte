<script lang="ts">
  import MapView from "$lib/Map.svelte";
  import IpSearchBox from "$lib/IpSearchBox.svelte";

  import {
    database,
    type LookupInfo,
    type Location,
    type Result,
  } from "../bindings";
  import { marker, Marker, type Map } from "leaflet";
  import { fade } from "svelte/transition";

  const SEARCH_ZOOM = 10;
  const regionNames = new Intl.DisplayNames(["en"], { type: "region" });

  let map: Map | null = $state(null);
  let result: { info: LookupInfo; ip: string } | string | null = $state(null);

  const search = async (input: Result<string, string> | null) => {
    if (input == null) {
      result = null;
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
  };

  let mrk: Marker = $state(marker({ lat: 0, lng: 0 }));

  $effect(() => {
    if (!map) return;

    if (result != null && typeof result == "object") {
      mrk.setLatLng(result.info.crd).addTo(map);

      if (map.getZoom() > SEARCH_ZOOM) {
        map.panTo(result.info.crd);
      } else {
        map.flyTo(result.info.crd, SEARCH_ZOOM, { duration: 1.5 });
      }
    } else {
      mrk.removeFrom(map);
    }
  });
</script>

<div class="flex grow">
  <MapView bind:map>
    <div class="rounded-box bg-base-300 absolute top-2 right-2 z-[999] border">
      <IpSearchBox {search} />
    </div>

    {#if typeof result == "string"}
      <p
        transition:fade={{ duration: 200 }}
        class="rounded-box bg-error absolute bottom-2 left-2 z-[999] p-2 text-sm select-none"
      >
        {result}
      </p>
    {:else if result != null && typeof result == "object"}
      {@render renderIpInfo(result.ip, result.info.loc)}
    {/if}
  </MapView>
</div>

{#snippet renderIpInfo(ip: string, loc: Location)}
  <div
    transition:fade={{ duration: 200 }}
    class="bg-base-200 rounded-box absolute right-2 bottom-2 z-[999] border p-2 text-right select-none"
  >
    <p class="underline">{ip}</p>
    <p class="text-sm">
      {`${loc.city ?? "Unknown City"}${loc.region ? `, ${loc.region}` : ""}`},
      {regionNames.of(loc.countryCode)}
    </p>
    {#await database.lookupDns(ip) then host}
      {#if host.status == "ok" && host.data != null}
        <p class="font-mono text-xs">DNS: {host.data}</p>
      {/if}
    {/await}
  </div>
{/snippet}
