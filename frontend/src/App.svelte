<script>
  import Settings from "./lib/Settings.svelte";
  import Toasts from "./lib/Toasts.svelte";
  import Map from "./lib/Map.svelte";
</script>

<Toasts />
<main class="p-4 space-y-4 w-screen h-screen flex flex-col">
  <Settings />
  <Map />
</main>
