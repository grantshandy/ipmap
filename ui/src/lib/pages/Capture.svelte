<script lang="ts">
  import MapView from "$lib/components/Map.svelte";

  import {
    database,
    renderDeviceName,
    newArc,
    movingAverageInfo,
    regionNames,
    humanFileSize,
    coordKey,
    type Pcap,
    type Coordinate,
    type CoordKey,
    type ActiveLocationRecord,
    updateArcStyle,
    CAPTURE_SHOW_ARCS,
    updateIcon,
  } from "$lib/bindings";
  import { marker, type Map } from "leaflet";
  import { onDestroy } from "svelte";

  const { pcap }: { pcap: Pcap } = $props();

  onDestroy(() => {
    pcap.stopCapture();
    pcap.unlisten();
  });

  let map: Map | null = $state(null);
  let focused: ActiveLocationRecord | null = $state(null);

  const coordinates: Record<string, CoordKey> = {};
  const locations: Record<CoordKey, ActiveLocationRecord> = {};

  let myLocation: Coordinate = { lat: 0, lng: 0 };
  database.myLocation().then((l) => {
    if (l) myLocation = l.crd;
  });

  pcap.connStart.on(async (ip, info) => {
    if (!map || !myLocation) return;

    const lookupResp = await database.lookupIp(ip);

    if (!lookupResp) {
      // TODO: handle case where location is not found
      console.warn(`${ip} not found in db`);
      return;
    }

    const locKey = coordKey(lookupResp.crd);
    coordinates[ip] = locKey;

    if (locKey in locations) {
      locations[locKey].ips[ip] = info;
    } else {
      locations[locKey] = {
        crd: locKey,
        ips: Object.fromEntries([[ip, info]]),
        marker: marker(lookupResp.crd)
          .on("click", () => setFocusedLocation(locKey))
          .addTo(map),
        arc: newArc(myLocation, lookupResp.crd, info, pcap.maxThroughput),
        location: lookupResp.loc,
      };

      if (CAPTURE_SHOW_ARCS) locations[locKey].arc.addTo(map);
    }

    updateIcon(locations[locKey], focused);
  });

  pcap.connEnd.on((ip: string) => {
    if (!coordinates[ip] || !map) return;

    const locRecord = locations[coordinates[ip]];
    if (!locRecord) return;

    delete locRecord.ips[ip];

    if (Object.keys(locRecord.ips).length == 0) {
      locRecord.marker.removeFrom(map);
      locRecord.arc.removeFrom(map);

      if (focused && focused.crd == locRecord.crd) {
        setFocusedLocation(null);
      }

      delete locations[coordinates[ip]];
      delete coordinates[ip];

      return;
    }

    updateIcon(locRecord, focused);
  });

  pcap.connUpdate.on(async (ip, info) => {
    const coord = coordinates[ip];
    if (!coord || !map) return;

    const loc = locations[coord];
    if (!loc) return;
    loc.ips[ip] = info;

    if (focused && focused.crd == coord) {
      focused.ips = loc.ips;
    }
  });

  pcap.updateEnd.on(async () => {
    for (const location of Object.values(locations)) {
      updateArcStyle(location, pcap.maxThroughput);
    }
  });

  // click off the icon to make it not focused
  $effect(() => {
    if (map) map.on("click", () => setFocusedLocation(null));
  });

  const setFocusedLocation = (key: CoordKey | null) => {
    if (!key) {
      if (focused) {
        const old = focused;
        focused = null;
        updateIcon(old, focused);
      }
      return;
    }

    const loc = locations[key];
    if (!loc || loc == focused) return;

    // reset previous location
    if (focused) {
      const prev = focused;
      focused = loc;
      updateIcon(prev, focused);
    }

    // update new location
    focused = loc;
    updateIcon(focused, focused);

    // center on location
    const zoom = (map?.getZoom() ?? 0) > 5 ? map?.getZoom() : 5;
    map?.flyTo(loc.marker.getLatLng(), zoom);
  };
</script>

<div class="flex grow">
  <MapView bind:map>
    <div
      class="join join-horizontal rounded-box absolute top-2 right-2 z-[999] border"
    >
      <select
        class="join-item select select-sm w-48"
        disabled={pcap.status.capture != null}
        bind:value={pcap.device}
      >
        {#each pcap.status.devices as device}
          <option value={device} disabled={!device.ready} selected>
            {#await renderDeviceName(device) then name}
              {name}
            {/await}
          </option>
        {/each}
      </select>

      {#if pcap.status.capture}
        <button
          onclick={pcap.stopCapture}
          class="join-item btn btn-sm btn-error"
        >
          Stop Capture
        </button>
      {:else}
        <button
          onclick={pcap.startCapture}
          class="join-item btn btn-sm btn-primary"
          disabled={pcap.device == null}
        >
          Start Capture
        </button>
      {/if}
    </div>

    {#if focused}
      {@render focusedLocationInfo(focused)}
    {/if}

    {#if pcap.status.capture != null}
      <div class="absolute right-2 bottom-2 z-[999] space-y-2">
        {#if pcap.session != null}
          <div
            class="bg-base-200 rounded-box flex divide-x border py-0.5 text-xs"
          >
            <span>&#8593; {humanFileSize(pcap.session.up.total)}</span>
            <span>&#8595; {humanFileSize(pcap.session.down.total)}</span>
          </div>
          <div
            class="bg-base-200 rounded-box flex divide-x border py-0.5 text-xs"
          >
            <span class="px-1"
              >&#8593; {humanFileSize(pcap.session.up.avgS)}/s</span
            >
            <span class="px-1"
              >&#8595; {humanFileSize(pcap.session.down.avgS)}/s</span
            >
          </div>
        {/if}
        <div class="bg-base-200 rounded-box flex divide-x border py-0.5">
          {@render directionIndicator("&#8593;", "--color-up")}
          {@render directionIndicator("&#8595;", "--color-down")}
          {@render directionIndicator("&#x2195;", "--color-mixed")}
        </div>
      </div>
    {/if}
  </MapView>
</div>

{#snippet directionIndicator(arrow: string, bgVar: string)}
  <div class="flex items-center px-2 py-0.5 text-center text-xs">
    <span
      style={`background-color: var(${bgVar});`}
      class="inline-block h-4 w-4 rounded-full">{@html arrow}</span
    >
  </div>
{/snippet}

{#snippet focusedLocationInfo(loc: ActiveLocationRecord)}
  <div
    class="bg-base-200 rounded-box absolute bottom-2 left-2 z-[999] max-h-120 w-54 space-y-3 overflow-y-scroll border p-2"
  >
    <p class="text-sm">
      {`${loc.location.city ?? "Unknown City"}${loc.location.region ? `, ${loc.location.region}` : ""}`},
      {regionNames.of(loc.location.countryCode)}
    </p>

    <div class="space-y-1">
      {#each Object.entries(loc.ips) as [ip, info], i}
        {#if i != 0}
          <hr />
        {/if}

        <h3>{ip}:</h3>
        <div class="font-mono text-xs">
          <p>&#8593; {movingAverageInfo(info.up)}</p>
          <p>&#8595; {movingAverageInfo(info.down)}</p>
        </div>
      {/each}
    </div>
  </div>
{/snippet}
