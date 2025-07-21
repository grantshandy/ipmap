<script lang="ts">
  import MapView from "$lib/components/Map.svelte";

  import {
    CAPTURE_SHOW_ARCS,
    database,
    renderDeviceName,
    newArc,
    movingAverageInfo,
    regionNames,
    humanFileSize,
    updateArc,
    newMarker,
    markerIcon,
    CaptureSession,
    type Pcap,
    type Coordinate,
    type CaptureLocation,
  } from "$lib/bindings";
  import { type Marker, type Map } from "leaflet";
  import { onDestroy } from "svelte";
  import type { GeodesicLine } from "leaflet.geodesic";

  const { pcap }: { pcap: Pcap } = $props();

  onDestroy(() => {
    pcap.stopCapture();
    pcap.unlisten();
  });

  // TODO: add back focused

  let map: Map | null = $state(null);
  let capture: CaptureSession | null = $state(null);

  type ArcMarker = { arc: GeodesicLine; marker: Marker };

  let locations: Record<string, ArcMarker> = {};

  // TODO: tie to backend further? return from startCapture?
  let myLocation: Coordinate = { lat: 0, lng: 0 };
  database.myLocation().then((l) => {
    if (l) myLocation = l.crd;
  });

  const onIpChanged = (crd: string, record: CaptureLocation | null) => {
    if (!capture || !map) return;

    // remove location
    if (record == null) {
      locations[crd].arc.remove();
      locations[crd].marker.remove();
      delete locations[crd];
      return;
    }

    // add new location
    if (!(crd in locations)) {
      const arc = newArc(myLocation, record.crd, record, capture.maxThroughput);

      if (CAPTURE_SHOW_ARCS) arc.addTo(map);

      locations[crd] = {
        marker: newMarker(record).addTo(map),
        arc,
      };
    } else {
      // new IP added to location

      // TODO: focused ?
      locations[crd].marker.setIcon(markerIcon(record, false));
    }
  };

  const onUpdate = (crd: string, loc: CaptureLocation) => {
    if (!capture) return;

    updateArc(loc, locations[crd].arc, capture.maxThroughput);
  };

  const onStopping = () => {
    for (const record of Object.values(locations)) {
      record.arc.remove();
      record.marker.remove();
    }

    locations = {};
  };

  const startCapture = () => {
    capture = pcap.startCapture(onUpdate, onIpChanged, onStopping);
  };

  const stopCapture = async () => {
    await pcap.stopCapture();
    capture = null;
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
        <button onclick={stopCapture} class="join-item btn btn-sm btn-error">
          Stop Capture
        </button>
      {:else}
        <button
          onclick={startCapture}
          class="join-item btn btn-sm btn-primary"
          disabled={pcap.device == null}
        >
          Start Capture
        </button>
      {/if}
    </div>

    <!-- {#if focused}
      {@render focusedLocationInfo(focused)}
    {/if} -->

    {#if capture != null}
      <div class="absolute right-2 bottom-2 z-[999] space-y-2">
        <div
          class="bg-base-200 rounded-box flex divide-x border py-0.5 text-xs"
        >
          <span>&#8593; {humanFileSize(capture.session.up.total)}</span>
          <span>&#8595; {humanFileSize(capture.session.down.total)}</span>
        </div>
        <div
          class="bg-base-200 rounded-box flex divide-x border py-0.5 text-xs"
        >
          <span class="px-1"
            >&#8593; {humanFileSize(capture.session.up.avgS)}/s</span
          >
          <span class="px-1"
            >&#8595; {humanFileSize(capture.session.down.avgS)}/s</span
          >
        </div>
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

{#snippet focusedLocationInfo(record: CaptureLocation)}
  <div
    class="bg-base-200 rounded-box absolute bottom-2 left-2 z-[999] max-h-120 w-54 space-y-3 overflow-y-scroll border p-2"
  >
    <p class="text-sm">
      {`${record.loc.city ?? "Unknown City"}${record.loc.region ? `, ${record.loc.region}` : ""}`},
      {regionNames.of(record.loc.countryCode)}
    </p>

    <div class="space-y-1">
      {#each Object.entries(record.ips) as [ip, info], i}
        {#if i != 0}
          <hr />
        {/if}

        <h3>{ip}:</h3>
        {#if info}
          <div class="font-mono text-xs">
            <p>&#8593; {movingAverageInfo(info.up)}</p>
            <p>&#8595; {movingAverageInfo(info.down)}</p>
          </div>
        {/if}
      {/each}
    </div>
  </div>
{/snippet}
