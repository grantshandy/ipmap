<script lang="ts">
  import "leaflet/dist/leaflet.css";
  import "leaflet";
  import "leaflet-edgebuffer";

  import { theme } from "../utils/theme";
  import { darkThemes } from "../themes.json";
  import { LatLngBounds, Map, map as mkMap, tileLayer } from "leaflet";
  import { DEFAULT_POS, DEFAULT_ZOOM } from "../map";

  export let map: Map;

  const mapAction = (cont: HTMLDivElement) => {
    map = mkMap(cont, {
      preferCanvas: false,
      minZoom: 2,
      maxZoom: 13,
      zoomControl: false,
      edgeBufferTiles: 1,
    } as any);
    map.setView(DEFAULT_POS, DEFAULT_ZOOM);
    map.attributionControl.remove();
    map.setMaxBounds(new LatLngBounds([-150, -300], [150, 400]));
    tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
      noWrap: true,
      edgeBufferTiles: 5,
    } as any).addTo(map);

    return {
      destroy: () => map.remove(),
    };
  };
</script>

<svelte:window on:resize={() => map.invalidateSize()} />

<div class="overflow-none relative grow rounded-box">
  {#if map}
    <div
      class="join select-bordered join-vertical absolute left-2 top-2 z-30 border"
    >
      <button
        on:click={() => map.zoomIn()}
        class="btn join-item btn-sm text-xl font-bold">+</button
      >
      <button
        on:click={() => map.zoomOut()}
        class="btn join-item btn-sm text-xl font-bold">&#x2212;</button
      >
    </div>
  {/if}
  <slot />
  <div
    use:mapAction
    class="z-20 h-full w-full select-none rounded-box"
    class:map-dark={darkThemes.includes($theme)}
  ></div>
</div>
