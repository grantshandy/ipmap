<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";

  import "ol/ol.css";
  import "../tailwind.css";

  import Map from "ol/Map.js";
  import OSM from "ol/source/OSM.js";
  import TileLayer from "ol/layer/Tile.js";
  import View from "ol/View.js";

  let map: Map | null = null;
  
  let locations: {
    ip: string;
    lat: number;
    lon: number;
    city: string;
    country: string;
  }[] = [];
  let connectionCount: number = 0;
  let uniqueIps: string[] = [];

  onMount(async () => {
    await listen("connection", async (event: any) => {
      let payload: { ip: string } = event.payload;

      connectionCount++;
      if (!uniqueIps.includes(payload.ip)) {
        uniqueIps.push(payload.ip);
      }
    });

    map = new Map({
      target: "map",
      layers: [
        new TileLayer({
          source: new OSM(),
        }),
      ],
      view: new View({
        center: [0, 0],
        zoom: 2,
      }),
    });
  });
</script>

<h1>look at me</h1>
<div class="w-screen h-screen flex">
  <div class="grow flex flex-col">
    {#if window.backendError}
      <div class="flow-root py-1 px-2 bg-red-600 text-gray-50 align-middle">
        <div class="float-left h-full flex items-center">
          <p><span class="font-bold">Error:</span> {window.backendError}</p>
        </div>
        <button class="float-right px-2 py-1 font-semibold border rounded-md"
          >Close</button
        >
      </div>
    {/if}
    <div class="grow" id="map" />
  </div>
</div>
