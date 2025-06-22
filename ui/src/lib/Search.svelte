<script lang="ts">
  import MapView from "$lib/Map.svelte";

  import { database, type LookupInfo, type Location } from "../bindings";
  import { marker, Marker, type Map } from "leaflet";
  import { Address4, Address6 } from "ip-address";
  import { fade } from "svelte/transition";

  const validDomainName =
    /^((?!-))(xn--)?[a-z0-9][a-z0-9-_]{0,61}[a-z0-9]{0,1}\.(xn--)?([a-z0-9\-]{1,61}|[a-z0-9-]{1,30}\.[a-z]{2,})$/;
  const SEARCH_ZOOM = 10;
  const regionNames = new Intl.DisplayNames(["en"], { type: "region" });

  let map: Map | null = $state(null);
  let input = $state("");
  let trimmedInput = $derived(input.replace(/\s/g, ""));

  let isDomainName: boolean = $derived(validDomainName.test(trimmedInput));
  let ipv4: string | null = $derived(
    database.ipv4Enabled && Address4.isValid(trimmedInput)
      ? new Address4(trimmedInput).correctForm()
      : null,
  );
  let ipv6: string | null = $derived(
    database.ipv6Enabled && Address6.isValid(trimmedInput)
      ? new Address6(trimmedInput).correctForm()
      : null,
  );

  let validInput: boolean = $derived(
    isDomainName || ipv4 != null || ipv6 != null,
  );

  let ip: string | null = $state(null);
  let result: LookupInfo | string | null = $state(null);

  $effect(() => {
    if (!validInput) {
      ip = null;
      result = null;
    }
  });

  const search = async () => {
    if (!validInput) return;

    if (isDomainName) {
      const lookup = await database.lookupHost(trimmedInput);

      if (lookup.status == "error" || lookup.data == null) {
        result = `No IP found for "${trimmedInput}"`;
        return;
      }

      ip = lookup.data;
    } else {
      ip = ipv4 ?? ipv6;
    }

    if (!ip) return;

    const lookup = await database.lookupIp(ip);

    if (!lookup) {
      result = `"${ip}" not found in database`;
      return;
    }

    input = trimmedInput;
    result = lookup;
  };

  let mrk: Marker = $state(marker({ lat: 0, lng: 0 }));

  $effect(() => {
    if (!map) return;

    if (result != null && typeof result == "object") {
      mrk.setLatLng(result.crd).addTo(map);

      if (map.getZoom() > SEARCH_ZOOM) {
        map.panTo(result.crd);
      } else {
        map.flyTo(result.crd, SEARCH_ZOOM, { duration: 1.5 });
      }
    } else {
      mrk.removeFrom(map);
    }
  });
</script>

<div class="flex grow">
  <MapView bind:map>
    <form
      class="join join-horizontal bg-base-300 rounded-box absolute top-2 right-2 z-[999] border select-none"
      onsubmit={search}
    >
      <input
        type="text"
        class="input input-sm join-item"
        placeholder="IP Address"
        oninput={() => {
          ip = null;
          result = null;
        }}
        class:input-error={trimmedInput.length != 0 && !validInput}
        bind:value={input}
      />
      <button
        class="btn btn-sm btn-primary join-item"
        disabled={!validInput || ip != null}
        type="submit">Search</button
      >
    </form>

    {#if typeof result == "string"}
      <p
        transition:fade={{ duration: 200 }}
        class="rounded-box bg-error absolute bottom-2 left-2 z-[999] p-2 text-sm select-none"
      >
        {result}
      </p>
    {:else if result != null && ip}
      {@render renderIpInfo(ip, result.loc)}
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
