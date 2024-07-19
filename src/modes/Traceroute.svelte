<script lang="ts">
  import MapView from "../components/MapView.svelte";
  import IpAddrInput from "../components/IpAddrInput.svelte";

  import { layerGroup, marker, type Map } from "leaflet";
  import { geoip, traceroute, type Coordinate } from "../bindings";
  import { database } from "../utils/database";
  import { mkIcon } from "../map";
  import { GeodesicLine } from "leaflet.geodesic";
  import IpView from "../components/IpView.svelte";

  let map: Map;

  let traceLayer = layerGroup();
  $: if (map) traceLayer.addTo(map);

  const myLocation = marker([0, 0], { icon: mkIcon(null, false) });
  $: if (map) myLocation.addTo(map);
  $: if ($database) geoip.myLocation().then((c) => myLocation.setLatLng(c));

  let error: string | null = null;
  let loading: boolean = false;

  let hops: { coord: Coordinate | null; ip: string }[] = [];

  const runTrace = async (ip: string) => {
    // reset arcs
    traceLayer.eachLayer((l) => l.remove());

    // load in trace
    loading = true;
    const hopsRecv = await traceroute.trace(ip);
    loading = false;

    let coords: Coordinate[] = [];
    coords[0] = myLocation.getLatLng();

    hops = [];
    for (let i = 0; i < hopsRecv.length; i++) {
      const loc = await geoip.lookupIp(hopsRecv[i]);

      hops[i] = {
        ip: hopsRecv[i],
        coord: loc,
      };

      if (!loc) {
        console.warn("no location found for", loc);
        return;
      }

      coords.push(loc);
    }

    for (let i = 1; i < coords.length; i++) {
      const prev = coords[i - 1];
      const curr = coords[i];

      // don't place markers on the same location
      if (prev.lat == curr.lat && prev.lng == curr.lng) continue;

      marker(curr, { icon: mkIcon(null, false) }).addTo(traceLayer);

      new GeodesicLine([prev, curr], {
        weight: 3,
        steps: 3,
        opacity: 0.75,
        className: "Outgoing",
      }).addTo(traceLayer);
    }
  };

  $: if (error || loading) {
    hops = [];
    traceLayer.eachLayer((l) => l.remove());
  }
</script>

<div class="flex grow space-x-2">
  <MapView bind:map>
    <div class="map-info-panel overflow-y-auto">
      <IpAddrInput bind:error disabled={loading} onSearch={runTrace} />

      {#if error}
        <p class="w-full text-center text-sm font-bold italic text-error">
          {error}
        </p>
      {/if}
      {#if loading}
        <p class="w-full text-center text-sm font-bold italic">Loading...</p>
      {/if}

      {#each hops as hop}
        <IpView ip={hop.ip} />
      {/each}
    </div>
  </MapView>
</div>
