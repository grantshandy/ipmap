<script lang="ts">
  import Map from "$lib/components/Map.svelte";
  import Globe from "$lib/components/Globe.svelte";

  import { type MapArgs, type MapInterface } from "$lib/map-interface.svelte";

  let {
    map = $bindable(),
    focused = $bindable(),
    globe,
    capture,
    children,
  }: {
    map: MapInterface | undefined;
    globe?: boolean;
  } & MapArgs = $props();
</script>

{#if globe}
  <Globe bind:this={map} {capture} {children} bind:focused />
{:else}
  <Map bind:this={map} {capture} {children} bind:focused />
{/if}
