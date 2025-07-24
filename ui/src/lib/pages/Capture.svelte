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
  } from "$lib/bindings";
  import { onDestroy } from "svelte";
  import { type MapInterface } from "$lib/map-interface.svelte";

  const UP_ARROW = "&#8593;";
  const DOWN_ARROW = "&#8595;";
  const MIXED_ARROW = "&#x2195;";

  const { pcap }: { pcap: Pcap } = $props();

  let map: MapInterface | undefined = $state();
  let focused: string | null = $state(null);
  let globe: boolean = $state(true);

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
    map.createMarker(crd, loc.crd, Object.keys(loc.ips).length);
    map.createArc(crd, myLocation, loc.crd, loc.thr, loc.dir);
  };

  export const locationRemoved = (crd: string) => {
    if (!map) return;
    map.removeMarker(crd);
    map.removeArc(crd);
  };

  export const update = async (crd: string, loc: CaptureLocation) => {
    if (!map) return;
    map.updateMarker(crd, loc.crd, Object.keys(loc.ips).length);
    map.updateArc(crd, myLocation, loc.crd, loc.thr, loc.dir);
  };
</script>

<GenericMap bind:map capture={pcap.capture} {globe} bind:focused>
  <div class="absolute top-2 right-2 z-[999] flex flex-col items-end space-y-2">
    {#if map}
      <CaptureStart
        bind:globe
        {pcap}
        callbacks={{
          locationAdded,
          locationRemoved,
          update,
        }}
      />
    {/if}

    {#if pcap.capture != null && CAPTURE_COLORS}
      <div
        class="bg-base-200 rounded-box flex divide-x border py-0.5 select-none"
      >
        {@render directionIndicator(UP_ARROW, "--color-up")}
        {@render directionIndicator(DOWN_ARROW, "--color-down")}
        {@render directionIndicator(MIXED_ARROW, "--color-mixed")}
      </div>
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

    {#if CAPTURE_SHOW_NOT_FOUND && pcap.capture.notFoundCount != 0}
      <div
        class="bg-base-200 rounded-box absolute top-2 left-14 z-[999] border p-1 text-xs"
      >
        <p>
          {pcap.capture.notFoundCount} IP{pcap.capture.notFoundCount > 1
            ? "s"
            : ""}
          not found in database
        </p>
      </div>
    {/if}

    <div
      class="bg-base-200 rounded-box absolute right-2 bottom-2 z-[999] w-45 space-y-2 border p-1 text-xs select-none"
    >
      {@render connectionStats(pcap.capture.session)}
    </div>
  {/if}
</GenericMap>

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
