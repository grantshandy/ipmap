<script lang="ts">
  import GenericMap from "$lib/components/GenericMap.svelte";

  import {
    Pcap,
    type Throughput,
    type CaptureLocation,
    type Connection,
    type Device,
  } from "tauri-plugin-pcap-api";
  import { PLATFORM } from "tauri-plugin-ipmap-api";

  import {
    renderLocationName,
    CAPTURE_SHOW_ARCS,
    CAPTURE_SHOW_MARKERS,
    CAPTURE_SHOW_NOT_FOUND,
  } from "$lib/utils";
  import { onDestroy } from "svelte";
  import { type MapComponent } from "$lib/page.svelte";

  // Library Functions:
  const UP_ARROW = "&#8593;";
  const DOWN_ARROW = "&#8595;";
  const MIXED_ARROW = "&#x2195;";

  export const humanFileSize = (size: number) => {
    const i = size == 0 ? 0 : Math.floor(Math.log(size) / Math.log(1024));
    return (
      +(size / Math.pow(1024, i)).toFixed(2) * 1 +
      " " +
      ["B", "kB", "MB", "GB", "TB"][i]
    );
  };

  export const throughputInfo = (info: Throughput): string =>
    `${humanFileSize(info.avgS)}/s | ${humanFileSize(info.total)}`;

  export const renderDeviceName = async (device: Device): Promise<string> => {
    if (PLATFORM === "windows") {
      return device.description ?? device.name;
    } else {
      return `${device.name}${device.description ? ": (" + device.description + ")" : ""}`;
    }
  };

  // Implementations:
  const { pcap }: { pcap: Pcap } = $props();

  let map: MapComponent | undefined = $state();
  let focused: string | null = $state(null);

  onDestroy(() => pcap.stopCapture());

  export const locationAdded = (crd: string, loc: CaptureLocation) => {
    if (!map) return;
    if (CAPTURE_SHOW_MARKERS)
      map.createMarker(crd, loc.crd, Object.keys(loc.ips).length);
    if (CAPTURE_SHOW_ARCS)
      map.createArc(crd, pcap.status.myLocation.crd, loc.crd, loc.thr, loc.dir);
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
      map.updateArc(crd, pcap.status.myLocation.crd, loc.crd, loc.thr, loc.dir);
  };
</script>

<GenericMap
  bind:map
  bind:focused
  capture={pcap.capture}
  {searchbox}
  {infobox}
/>

{#snippet searchbox()}
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
      onclick={() => pcap.stopCapture()}
      class="join-item btn btn-sm btn-error"
    >
      Stop Capture
    </button>
  {:else}
    <button
      onclick={() =>
        pcap.startCapture({
          locationAdded,
          locationRemoved,
          update,
        })}
      class="join-item btn btn-sm btn-primary"
      disabled={pcap.device == null}
    >
      Start Capture
    </button>
  {/if}
{/snippet}

{#snippet infobox()}
  <div class="flex flex-col items-end space-y-2 p-2">
    {#if pcap.capture != null}
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

      <div
        class="bg-base-200 rounded-box w-45 space-y-2 border p-1 text-xs select-none"
      >
        {@render connectionStats(pcap.capture.session)}
      </div>

      {#if focused}
        <div
          class="bg-base-200 rounded-box max-h-120 w-54 space-y-3 overflow-y-scroll border p-2"
        >
          {@render focusedInfo(pcap.capture.connections[focused])}
        </div>
      {/if}
    {/if}
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
