<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api";

  import Map from "ol/Map.js";
  import OSM from "ol/source/OSM.js";
  import TileLayer from "ol/layer/Tile.js";
  import View from "ol/View.js";
  import { onDestroy, onMount } from "svelte";

  listen("error", (event) => {
    error = event.payload as string;
  });

  let error;
  let map;

  onMount(() => {
    invoke("poll_connections");

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

<main class="w-screen h-screen flex">
  <div class="grow">
    {#if error}
      <div class="w-full bg-red-500 text-slate-50 p-4 flow-root">
        <div class="float-left h-full flex items-center">
          <p>Error: <span class="font-bold">{error}</span></p>
        </div>
        <button class="float-right px-2 py-1 rounded-md border-2 hover:bg-red-600">Close</button>
      </div>
    {/if}
    <div id="map" class="w-full h-full" />
  </div>
</main>
