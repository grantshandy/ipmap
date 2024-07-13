<script lang="ts">
  import "leaflet/dist/leaflet.css";
  import "leaflet";
  import "leaflet-active-area";
  import "leaflet-edgebuffer";

  import { darkTheme, theme } from "../stores/theme";
  import { Map, map as mkMap, tileLayer } from "leaflet";
  import { DEFAULT_POS, DEFAULT_ZOOM } from "../map";

  export let map: Map;

  const mapAction = (cont: HTMLDivElement) => {
    map = mkMap(cont, {
      preferCanvas: false,
      minZoom: 2,
      maxZoom: 13,
      zoomControl: false,
    });
    map.setView(DEFAULT_POS, DEFAULT_ZOOM);
    map.setActiveArea(cont);
    tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
      attribution: "&copy; OSM Contributors",
      noWrap: true,
      edgeBufferTiles: 5,
    } as any).addTo(map);

    return {
      destroy: () => map.remove(),
    };
  };
</script>

<svelte:window on:resize={() => map.invalidateSize()} />

<div class="overflow-none relative grow select-none rounded-box">
  {#if map}
    <div class="join join-vertical absolute left-2 top-2 z-30">
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
    class="z-20 h-full w-full rounded-box"
    class:map-dark={$theme == darkTheme}
  ></div>
</div>
