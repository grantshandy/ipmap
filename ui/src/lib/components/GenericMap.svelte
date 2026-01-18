<script lang="ts">
  import Map from "$lib/components/Map.svelte";
  import Globe from "$lib/components/Globe.svelte";
  import PageSelector from "./PageSelector.svelte";

  import { openAboutWindow } from "tauri-plugin-ipmap-api";

  import { pageState, type MapArgs, type MapComponent } from "$lib/page.svelte";
  import type { Snippet } from "svelte";

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
    }
  };
</script>

<div class="relative grow">
  <div
    class="join join-horizontal bg-base-200 border-btn rounded-field absolute top-2 left-2 z-999 flex"
  >
    <PageSelector />
    {@render searchbox?.()}
  </div>

  <div class="absolute right-0 bottom-0 z-999">
    {@render infobox?.()}
  </div>

  <div class="absolute top-2 right-2 z-999 flex flex-col space-y-2 select-none">
    <div class="join join-vertical">
      <button
        onclick={() => map?.zoomIn()}
        class="btn join-item btn-sm border-btn border-b-0 text-xl font-bold"
        >+</button
      >
      <button
        onclick={() => map?.zoomOut()}
        class="btn join-item btn-sm border-btn border-t-0 text-xl font-bold"
        >&#x2212;</button
      >
    </div>

    <button class="btn btn-sm border-btn w-full" onclick={openAboutWindow}>
      ?
    </button>

    <button
      class="btn btn-sm border-btn w-full"
      class:btn-primary={pageState.globe}
      onclick={() => (pageState.globe = !pageState.globe)}
    >
      3D
    </button>
  </div>

  {#if pageState.globe}
    <Globe bind:this={map} {capture} bind:focused />
  {:else}
    <Map bind:this={map} {capture} bind:focused />
  {/if}
</div>
