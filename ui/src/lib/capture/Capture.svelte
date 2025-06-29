<script lang="ts">
  import { Pcap, database, type Coordinate } from "$lib/../bindings";
  import MapView from "$lib/Map.svelte";
  import { divIcon, type Map, marker, type Marker } from "leaflet";
  import { GeodesicLine } from "leaflet.geodesic";
  import { onDestroy } from "svelte";

  const { pcap }: { pcap: Pcap } = $props();

  onDestroy(() => {
    pcap.stopCapture();
  });

  let map: Map | null = $state(null);

  type CoordKey = string;
  type ActiveLocationRecord = {
    ips: string[];
    marker: Marker;
    arc: GeodesicLine;
  };

  const coordKey = (p: Coordinate): CoordKey => `${p.lat}${p.lng}`;

  const updateLocation = (loc: ActiveLocationRecord) => {
    loc.marker.setIcon(
      divIcon({
        html: `<div>${loc.ips.length}</div>`,
        className: "marker-icon",
        iconSize: [20, 20],
        iconAnchor: [10, 10],
      }),
    );
    loc.marker.bindPopup(loc.ips.map((ip) => `<p>${ip}</p>`).join(""));
  };

  const keys: Record<string, CoordKey> = {};
  const locs: Record<CoordKey, ActiveLocationRecord> = {};

  let myLocation: Coordinate | null = null;
  database.myLocation().then((l) => {
    if (l) {
      myLocation = l.crd;
    }
  });

  pcap.conn.onStart(async (ip) => {
    if (!map || !myLocation) return;

    const lookupResp = await database.lookupIp(ip);

    if (!lookupResp) {
      // TODO: handle case where location is not found
      console.warn(`${ip} not found in db`);
      return;
    }

    const locKey = coordKey(lookupResp.crd);
    keys[ip] = locKey;

    if (locKey in locs) {
      locs[locKey].ips.push(ip);
    } else {
      locs[locKey] = {
        ips: [ip],
        marker: marker(lookupResp.crd).addTo(map),
        arc: new GeodesicLine([lookupResp.crd, myLocation]).addTo(map),
      };
    }

    updateLocation(locs[locKey]);
  });

  pcap.conn.onEnd((ip) => {
    if (!keys[ip] || !map) return;

    const locRecord = locs[keys[ip]];
    if (!locRecord) return;

    const idx = locRecord.ips.indexOf(ip);
    if (idx > -1) locRecord.ips.splice(idx, 1);

    if (locRecord.ips.length == 0) {
      locRecord.marker.removeFrom(map);
      locRecord.arc.removeFrom(map);
      delete locs[keys[ip]];
      delete keys[ip];
    } else {
      updateLocation(locRecord);
    }
  });
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
  </MapView>
</div>
