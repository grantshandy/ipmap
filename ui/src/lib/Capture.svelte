<script lang="ts">
  import { Pcap, database, type Coordinate } from "$lib/../bindings";
  import MapView from "$lib/Map.svelte";
  import { type Map, marker, type Marker } from "leaflet";
  import { onDestroy } from "svelte";

  const { pcap }: { pcap: Pcap } = $props();

  onDestroy(() => {
    pcap.stopCapture();
  });

  let map: Map | null = $state(null);

  type CoordKey = string;
  const coordKey = (p: Coordinate): CoordKey => `${p.lat}${p.lng}`;
  const popup = (ips: string[]): string =>
    ips.map((ip) => `<p>${ip}</p>`).join("");

  const keys: Record<string, CoordKey> = {};
  const locs: Record<CoordKey, { ips: string[]; marker: Marker }> = {};

  pcap.conn.onStart(async (ip) => {
    if (!map) return;

    const loc = await database.lookupIp(ip);

    if (!loc) {
      // TODO: handle case where location is not found
      return;
    }

    const key = coordKey(loc.crd);

    keys[ip] = key;

    if (key in locs) {
      const marker = locs[key];

      marker.ips.push(ip);
      marker.marker.bindPopup(popup(marker.ips));
    } else {
      locs[key] = {
        ips: [ip],
        marker: marker(loc.crd)
          .bindPopup(popup([ip]))
          .addTo(map),
      };
    }
  });

  pcap.conn.onEnd((ip) => {
    if (!keys[ip]) return;

    const marker = locs[keys[ip]];

    const idx = marker.ips.indexOf(ip);
    if (idx > -1) marker.ips.splice(idx, 1);

    if (marker.ips.length == 0) {
      marker.marker.remove();
      delete locs[ip];
      delete keys[ip];
    } else {
      marker.marker.bindPopup(popup(marker.ips));
    }
  });
</script>

<div class="flex grow flex-col space-y-3">
  <div class="flow-root w-full">
    <div class="join join-horizontal float-left">
      <select
        class="select select-sm join-item"
        disabled={pcap.status.capture != null}
        bind:value={pcap.device}
      >
        {#each pcap.status.devices as device}
          <option value={device} disabled={!device.ready} selected>
            {device.name}
            {#if device.description}
              : ({device.description})
            {/if}
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
    {#if pcap.status.capture != null}
      <p class="float-right">
        No. Connections: {Object.keys(pcap.connections).length}
      </p>
    {/if}
  </div>

  <div class="flex grow">
    <MapView bind:map>
      <div
        class="bg-base-300 absolute top-0 right-0 bottom-0 z-[999] w-64 opacity-50"
      >
        <!-- <p>On Map</p>
                {@render showConnections(onMap)}
                <p>Not on Map</p>
                {@render showConnections(notOnMap)} -->
      </div>
    </MapView>
  </div>
</div>

{#snippet showConnections(connections: string[])}
  <ul>
    {#each connections as ip}
      <li>{ip}</li>
    {/each}
  </ul>
{/snippet}
