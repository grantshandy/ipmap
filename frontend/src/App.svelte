<script lang="ts">
  import Settings from "./lib/Settings.svelte";
  import Toasts from "./lib/Toasts.svelte";
  import Map from "./lib/Map.svelte";
  import { type DatabaseInfo } from "./utils";

  let database: DatabaseInfo | null = null;
  let capturing: string | null = null;
</script>

<Toasts />
<main class="p-4 space-y-4 w-screen h-screen flex flex-col">
  <Settings bind:database={database} bind:capturing={capturing} />
  <Map bind:database={database} bind:capturing={capturing} />
</main>
