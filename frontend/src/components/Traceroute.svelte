<script lang="ts">
  import MapView from "./MapView.svelte";
  import IpAddrInput from "./IpAddrInput.svelte";

  import { layerGroup, marker, type Map } from "leaflet";
  import { geoip, traceroute, type Coordinate } from "../bindings";
  import { database } from "../stores/database";
  import { mkIcon } from "../map";
  import { GeodesicLine } from "leaflet.geodesic";

  let map: Map;

  let traceLayer = layerGroup();
  $: if (map) traceLayer.addTo(map);

  const myLocation = marker([0, 0], { icon: mkIcon(null, false) });
  $: if (map) myLocation.addTo(map);
  $: if ($database) geoip.myLocation().then((c) => myLocation.setLatLng(c));

  let ip: string | null = null;
  let error: string | null = null;
  let loading: boolean = false;

  const runTrace = async () => {
    if (!ip) return;

    // reset arcs
    traceLayer.eachLayer((l) => l.remove());

    // load in trace
    loading = true;
    const hops = await traceroute.trace(ip);
    loading = false;

    let coords: Coordinate[] = [];
    coords[0] = myLocation.getLatLng();

    for (const hop of hops) {
      const loc = await geoip.lookupIp(hop);

      if (!loc) {
        console.warn("no location found for", loc);
        return;
      }

      coords.push(loc);
    }

    for (let i = 1; i < coords.length; i++) {
      marker(coords[i], { icon: mkIcon(null, false) }).addTo(traceLayer);

      new GeodesicLine([coords[i - 1], coords[i]], {
        weight: 3,
        steps: 3,
        opacity: 0.75,
        className: "Outgoing",
      }).addTo(traceLayer);
    }
  };
</script>

<div class="flex grow space-x-2">
  <MapView bind:map>
    <div
      class="rounded-bl-box bg-base-200/[0.8] absolute right-0 top-0 z-30 w-1/4 space-y-3 p-2"
    >
      <div class="flex items-center">
        <IpAddrInput bind:ip bind:error disabled={loading} />
        <button
          class="btn btn-sm btn-primary"
          disabled={error != null || !ip || loading}
          on:click={runTrace}>Search</button
        >
      </div>

      {#if error}
        <p class="text-error w-full text-center text-sm font-bold italic">
          {error}
        </p>
      {/if}
      {#if loading}
        <p class="w-full text-center text-sm font-bold italic">Loading...</p>
      {/if}

      <!-- show info -->
    </div>
  </MapView>
</div>
