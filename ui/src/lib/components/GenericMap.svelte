<script lang="ts">
  import Map from "$lib/components/Map.svelte";
  import Globe from "$lib/components/Globe.svelte";

  import { pageState, type MapArgs, type MapComponent } from "$lib/page.svelte";
  import PageSelector from "./PageSelector.svelte";
  import type { Snippet } from "svelte";
  import { openAboutWindow } from "$lib/bindings";

  let {
    map = $bindable(),
    focused = $bindable(),
    searchbox,
    infobox,
    capture,
  }: {
    map: MapComponent | undefined;
    globe?: boolean;
    searchbox?: Snippet;
    infobox?: Snippet;
  } & MapArgs = $props();

  document.onkeydown = (ev) => {
    switch (ev.key) {
      case "=":
      case "+":
        map?.zoomIn();
        break;
      case "-":
        map?.zoomOut();
        break;
      case "3":
        pageState.globe = !pageState.globe;
        break;
    }
  };
</script>

<div class="relative grow">
  <div class="join join-horizontal absolute top-0 left-0 z-999 flex p-2">
    <PageSelector />
    {@render searchbox?.()}
  </div>

  <div class="absolute right-0 bottom-0 z-999 p-2">
    {@render infobox?.()}
  </div>

  <div
    class="absolute top-0 right-0 z-999 flex flex-col items-end space-y-2 p-2 select-none"
  >
    <div class="join join-vertical">
      <button
        onclick={() => map?.zoomIn()}
        class="btn join-item btn-sm border-b-0 border-white text-xl font-bold"
        >+</button
      >
      <button
        onclick={() => map?.zoomOut()}
        class="btn join-item btn-sm border-t-0 border-white text-xl font-bold"
        >&#x2212;</button
      >
    </div>

    <button class="btn btn-sm text-xl" onclick={openAboutWindow}>?</button>

    <div
      class="bg-base-200 rounded-box flex items-center space-x-2 border border-white px-2 py-1 text-sm"
    >
      <label for="globe" class="flex h-full items-center text-xs">3D</label>

      <input
        id="globe"
        type="checkbox"
        bind:checked={pageState.globe}
        class="toggle toggle-xs"
      />
    </div>
  </div>

  {#if pageState.globe}
    <Globe bind:this={map} {capture} bind:focused />
  {:else}
    <Map bind:this={map} {capture} bind:focused />
  {/if}
</div>
