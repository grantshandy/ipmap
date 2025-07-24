<script lang="ts">
  import Map from "$lib/components/Map.svelte";
  import Globe from "$lib/components/Globe.svelte";

  import { pageState, type MapArgs, type MapComponent } from "$lib/page.svelte";

  let {
    map = $bindable(),
    focused = $bindable(),
    capture,
    children,
  }: {
    map: MapComponent | undefined;
    globe?: boolean;
  } & MapArgs = $props();

  // TODO: sync map zoom
</script>

<div class="relative grow">
  {#if pageState.globe}
    <Globe bind:this={map} {capture} {children} bind:focused />
  {:else}
    <Map bind:this={map} {capture} {children} bind:focused />
  {/if}
</div>
