<script lang="ts">
  import { database } from "./stores/database";

  import Capture from "./components/Capture.svelte";
  import DatabaseSelector from "./components/DatabaseSelector.svelte";
  import Search from "./components/Search.svelte";

  let state: "search" | "capture" = "search";
</script>

<main class="w-screen min-h-screen p-2 space-y-3 flex flex-col">
  <div class="flow-root space-x-3">
    <select bind:value={state} class="select select-sm select-bordered">
      <option value="search">Search</option>
      <option value="capture">Capture</option>
    </select>
    <div class="float-right">
      <DatabaseSelector />
    </div>
  </div>
  <hr />
  {#if $database}
    {#if state == "search"}
      <Search />
    {:else if state == "capture"}
      <Capture />
    {/if}
  {:else}
    <p>Load an IP-Geo Database to start</p>
  {/if}
</main>
