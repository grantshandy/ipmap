<script lang="ts">
  import MapView from "./MapView.svelte";
  import IpAddrInput from "./IpAddrInput.svelte";
  import LocationInfoView from "./LocationInfoView.svelte";
  import IpView from "./IpView.svelte";

  import { layerGroup, marker, type Map } from "leaflet";
  import {
    geoip,
    traceroute,
    type Coordinate,
    type TracerouteOptions,
    type Hop,
  } from "../bindings";
  import { database } from "../utils/database";
  import { mkIcon } from "../map";
  import { GeodesicLine } from "leaflet.geodesic";

  let map: Map;

  let traceLayer = layerGroup();
  $: if (map) traceLayer.addTo(map);

  const myLocation = marker([0, 0], { icon: mkIcon(null, false) });
  $: if (map) myLocation.addTo(map);
  $: if ($database) geoip.myLocation().then((c) => myLocation.setLatLng(c));

  let error: string | null = null;
  let loading: boolean = false;

  let options: TracerouteOptions = localStorage.tracerouteOptions
    ? JSON.parse(localStorage.tracerouteOptions)
    : traceroute.defaultOptions();
  $: localStorage.tracerouteOptions = JSON.stringify(options);

  let hops: Hop[] = [];

  const runTrace = async (ip: string) => {
    loading = true;
    hops = [];
    hops = await traceroute.trace(ip, options);
    loading = false;
    viewFlow();
  };

  const viewFlow = () => {
    traceLayer.eachLayer((l) => l.remove());

    const locations: Coordinate[] = hops
      .filter((hop) => hop.coord != null)
      .map((hop) => hop.coord as Coordinate);

    for (let i = 1; i < locations.length; i++) {
      const prev = locations[i - 1];
      const curr = locations[i];

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

      <div class="space-y-1">
        <div
          class="flex items-center rounded-box border border-base-300 bg-base-100 px-2 py-1 text-xs"
        >
          <p class="grow select-none">Max Rounds</p>
          <input
            bind:value={options.maxRounds}
            class="input input-xs input-bordered"
            type="number"
            min="1"
            max="255"
          />
        </div>
        <div
          class="flex items-center rounded-box border border-base-300 bg-base-100 px-2 py-1 text-xs"
        >
          <p class="grow select-none">Max Time to Live</p>
          <input
            bind:value={options.maxTtl}
            class="input input-xs input-bordered"
            type="number"
            min="1"
            max="255"
          />
        </div>
      </div>

      {#if error}
        <p
          class="w-full select-none text-center text-sm font-bold italic text-error"
        >
          {error}
        </p>
      {/if}

      {#if loading}
        <p class="w-full select-none text-center text-sm font-bold italic">
          Loading...
        </p>
      {/if}

      {#if hops.length > 0}
        <hr class="bg-base-100 text-base-100" />

        <h2
          class="select-none rounded-box border border-base-300 bg-base-100 px-3 py-2 text-lg font-bold"
        >
          {hops.length} Hops:
        </h2>

        {#each hops as hop, i}
          <div
            class="space-y-3 rounded-box border border-base-300 bg-base-100 px-3 py-2"
          >
            <h3 class="font-semibold">
              {i + 1}:
              {#if hop.ip}
                {#if hop.coord}
                  <button
                    class="link"
                    on:click={() => map.flyTo(hop.coord ?? [0, 0], 6)}
                  >
                    {hop.ip}
                  </button>
                {:else}
                  {hop.ip}
                {/if}
              {:else}
                <span class="text-sm font-normal italic"
                  >(No address found for hop)</span
                >
              {/if}
            </h3>

            {#if hop.coord}
              <LocationInfoView coord={hop.coord} />
            {/if}

            {#if hop.ip}
              <IpView ip={hop.ip} />
            {/if}
          </div>
        {/each}
      {/if}
    </div>
  </MapView>
</div>
