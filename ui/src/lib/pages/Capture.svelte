<script lang="ts">
  import CaptureStart from "$lib/components/CaptureStart.svelte";
  import GenericMap from "$lib/components/GenericMap.svelte";

  import {
    CAPTURE_COLORS,
    CAPTURE_SHOW_NOT_FOUND,
    database,
    renderLocationName,
    throughputInfo,
    type Pcap,
    type CaptureLocation,
    type Connection,
    CAPTURE_SHOW_ARCS,
    CAPTURE_SHOW_MARKERS,
  } from "$lib/bindings";
  import { onDestroy } from "svelte";
  import { type MapComponent } from "$lib/page.svelte";
  import GlobeSwitcher from "$lib/components/GlobeSwitcher.svelte";

  const UP_ARROW = "&#8593;";
  const DOWN_ARROW = "&#8595;";
  const MIXED_ARROW = "&#x2195;";

  const { pcap }: { pcap: Pcap } = $props();

  let map: MapComponent | undefined = $state();
  let focused: string | null = $state(null);

  onDestroy(() => pcap.stopCapture());

  // TODO: tie to backend further? return from startCapture?
  let myLocation = $state({ lat: 0, lng: 0 });
  database.myLocation().then((l) => {
    if (l.status == "ok") {
      myLocation = l.data.crd;
    } else {
      console.warn(l.error);
    }
  });

  export const locationAdded = (crd: string, loc: CaptureLocation) => {
    if (!map) return;
    if (CAPTURE_SHOW_MARKERS)
      map.createMarker(crd, loc.crd, Object.keys(loc.ips).length);
    if (CAPTURE_SHOW_ARCS)
      map.createArc(crd, myLocation, loc.crd, loc.thr, loc.dir);
  };

  export const locationRemoved = (crd: string) => {
    if (!map) return;
    if (CAPTURE_SHOW_MARKERS) map.removeMarker(crd);
    if (CAPTURE_SHOW_ARCS) map.removeArc(crd);
  };

  export const update = async (crd: string, loc: CaptureLocation) => {
    if (!map) return;
    if (CAPTURE_SHOW_MARKERS)
      map.updateMarker(crd, loc.crd, Object.keys(loc.ips).length);
    if (CAPTURE_SHOW_ARCS)
      map.updateArc(crd, myLocation, loc.crd, loc.thr, loc.dir);
  };
</script>

<GenericMap bind:map capture={pcap.capture} bind:focused>
  <div class="absolute top-2 right-2 z-[999] flex flex-col items-end space-y-2">
    <div class="flex items-center space-x-2">
      <GlobeSwitcher />
      <CaptureStart
        {pcap}
        callbacks={{
          locationAdded,
          locationRemoved,
          update,
        }}
      />
    </div>

    {#if pcap.capture != null}
      <div
        class="bg-base-200 rounded-box w-45 space-y-2 border p-1 text-xs select-none"
      >
        {@render connectionStats(pcap.capture.session)}
      </div>

      {#if CAPTURE_SHOW_NOT_FOUND && pcap.capture.notFoundCount != 0}
        <div class="bg-base-200 rounded-box border p-1 text-xs">
          <p>
            {pcap.capture.notFoundCount} IP{pcap.capture.notFoundCount > 1
              ? "s"
              : ""}
            not found in database
          </p>
        </div>
      {/if}
    {/if}
  </div>

  {#if pcap.capture != null}
    {#if focused}
      <div
        class="bg-base-200 rounded-box absolute bottom-2 left-2 z-[999] max-h-120 w-54 space-y-3 overflow-y-scroll border p-2"
      >
        {@render focusedInfo(pcap.capture.connections[focused])}
      </div>
    {/if}
  {/if}
</GenericMap>

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
