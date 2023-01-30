<script lang="ts">
  import Map from "ol/Map.js";
  import OSM from "ol/source/OSM.js";
  import TileLayer from "ol/layer/Tile.js";
  import View from "ol/View.js";
  import { onMount } from "svelte";

  onMount(() => {
    new Map({
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
  <div id="map" class="grow" />
</main>
