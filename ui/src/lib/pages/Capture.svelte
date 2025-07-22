<script lang="ts">
  import MapView from "$lib/components/Map.svelte";
  import CaptureStart from "$lib/components/CaptureStart.svelte";

  import {
    CAPTURE_SHOW_ARCS,
    CAPTURE_COLORS,
    CAPTURE_SHOW_NOT_FOUND,
    database,
    renderLocationName,
    throughputInfo,
    CaptureSession,
    type Pcap,
    type Coordinate,
    type CaptureLocation,
    type Connection,
  } from "$lib/bindings";
  import { type Marker, type Map } from "leaflet";
  import { onDestroy } from "svelte";
  import type { GeodesicLine } from "leaflet.geodesic";
  import { markerIcon, newArc, newMarker, updateArc } from "$lib/leaflet-utils";

  const UP_ARROW = "&#8593;";
  const DOWN_ARROW = "&#8595;";
  const MIXED_ARROW = "&#x2195;";

  const { pcap }: { pcap: Pcap } = $props();

  onDestroy(() => {
    pcap.stopCapture();
  });

  let map: Map | null = $state(null);
  let capture: CaptureSession | null = $state(null);
  let focused: string | null = $state(null);

  type ArcMarker = { arc: GeodesicLine; marker: Marker };

  let locations: Record<string, ArcMarker> = {};

  // TODO: tie to backend further? return from startCapture?
  let myLocation: Coordinate = { lat: 0, lng: 0 };
  database.myLocation().then((l) => {
    if (l.status == "ok") {
      myLocation = l.data.crd;
    } else {
      console.warn(l.error);
    }
  });

  $effect(() => {
    if (map) map.on("click", () => setFocused(null));
  });

  const setFocused = (crd: string | null) => {
    if (!capture) return;

    if (focused && focused in capture.connections)
      locations[focused].marker.setIcon(
        markerIcon(capture.connections[focused], false),
      );

    focused = crd && crd in locations ? crd : null;
    if (!focused) return;

    locations[focused].marker.setIcon(
      markerIcon(capture.connections[focused], true),
    );
  };

  const onIpChanged = (crd: string, record: CaptureLocation | null) => {
    if (!capture || !map) return;

    // remove location
    if (record == null) {
      if (focused == crd) setFocused(null);

      if (crd in locations) {
        locations[crd].arc.remove();
        locations[crd].marker.remove();
      }

      delete locations[crd];
      return;
    }

    if (crd in locations) {
      // IP generally added/removed
      locations[crd].marker.setIcon(markerIcon(record, focused == crd));
    } else {
      // add new location
      const arc = newArc(myLocation, record.crd, record, capture.maxThroughput);

      if (CAPTURE_SHOW_ARCS) arc.addTo(map);

      locations[crd] = {
        marker: newMarker(record)
          .on("click", () => setFocused(crd))
          .addTo(map),
        arc,
      };
    }
  };

  const onUpdate = async (crd: string, loc: CaptureLocation) => {
    if (capture && crd in locations)
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
      class="absolute top-2 right-2 z-[999] flex flex-col items-end space-y-2"
    >
      <CaptureStart {pcap} {startCapture} {stopCapture} />

      {#if capture != null && CAPTURE_COLORS}
        <div
          class="bg-base-200 rounded-box flex divide-x border py-0.5 select-none"
        >
          {@render directionIndicator(UP_ARROW, "--color-up")}
          {@render directionIndicator(DOWN_ARROW, "--color-down")}
          {@render directionIndicator(MIXED_ARROW, "--color-mixed")}
        </div>
      {/if}
    </div>

    {#if capture != null}
      {#if focused}
        <div
          class="bg-base-200 rounded-box absolute bottom-2 left-2 z-[999] max-h-120 w-54 space-y-3 overflow-y-scroll border p-2"
        >
          {@render focusedInfo(capture.connections[focused])}
        </div>
      {/if}

      {#if CAPTURE_SHOW_NOT_FOUND && capture.notFoundCount != 0}
        <div
          class="bg-base-200 rounded-box absolute top-2 left-14 z-[999] border p-1 text-xs"
        >
          <p>
            {capture.notFoundCount} IP{capture.notFoundCount > 1 ? "s" : ""}
            not found in database
          </p>
        </div>
      {/if}

      <div
        class="bg-base-200 rounded-box absolute right-2 bottom-2 z-[999] w-45 space-y-2 border p-1 text-xs select-none"
      >
        {@render connectionStats(capture.session)}
      </div>
    {/if}
  </MapView>
</div>

{#snippet directionIndicator(arrow: string, bgVar: string)}
  <div class="flex items-center px-2 py-0.5 text-center text-xs">
    <span
      style={`background-color: var(${bgVar});`}
      class="inline-block h-4 w-4 rounded-full"
    >
      {@html arrow}
    </span>
  </div>
{/snippet}

{#snippet focusedInfo(record: CaptureLocation)}
  <p class="text-sm">{renderLocationName(record.loc)}</p>

  <div class="space-y-1">
    {#each Object.entries(record.ips) as [ip, conn], i}
      {#if i != 0}
        <hr />
      {/if}

      <h3>{ip}:</h3>
      {#if conn}
        <div class="text-xs">
          {@render connectionStats(conn)}
        </div>
      {/if}
    {/each}
  </div>
{/snippet}

{#snippet connectionStats(conn: Connection)}
  <p>
    {@html UP_ARROW}
    {throughputInfo(conn.up)}
  </p>
  <p>
    {@html DOWN_ARROW}
    {throughputInfo(conn.down)}
  </p>
{/snippet}
