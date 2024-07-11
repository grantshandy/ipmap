<script lang="ts">
  import "leaflet/dist/leaflet.css";
  import { map } from "../stores/map";
  import { darkTheme, theme } from "../stores";

  const mapAction = (cont: HTMLDivElement) => {
    map.init(cont);

    return {
      destroy: () => map.deinit(),
    };
  };
</script>

<div class="relative grow select-none rounded-box">
  {#if $map}
    <div class="join join-vertical absolute left-2 top-2 z-30">
      <button
        on:click={() => $map.inst.zoomIn()}
        class="btn join-item btn-sm text-xl font-bold">+</button
      >
      <button
        on:click={() => $map.inst.zoomOut()}
        class="btn join-item btn-sm text-xl font-bold">&#x2212;</button
      >
    </div>
  {/if}
  <slot />
  <div
    use:mapAction
    class="z-20 h-full w-full"
    class:map-dark={$theme == darkTheme}
  ></div>
</div>

<style lang="postcss">
</style>
