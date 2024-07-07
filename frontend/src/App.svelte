<script lang="ts">
  import { database } from "./stores/database";
  import { fade } from "svelte/transition";

  import Capture from "./components/Capture.svelte";
  import DatabaseSelector from "./components/DatabaseSelector.svelte";
  import Search from "./components/Search.svelte";
  import { open } from "@tauri-apps/api/shell";

  let state: "search" | "capture" = "capture";
  let loading: string | null = "Internal Database";
</script>

{#if !$database}
  <main
    transition:fade={{ duration: 200 }}
    class="page flex items-center justify-center"
  >
    <div class="text-center space-y-9 select-none">
      {#if !loading}
        <h1 class="font-bold text-2xl">
          Load an IP-Geolocation Database
        </h1>
      {/if}
      <DatabaseSelector bind:loading />
      {#if !loading}
        <p class="max-w-sm mx-auto leading-loose">
          Databases must be in the <span
            class="code"
            >*-city-ipv4-num.csv</span
          >
          format, and can be found at the
          <button
            on:click={() => open("https://github.com/sapics/ip-location-db")}
            class="text-success underline">ip-location-db</button
          > repository.
        </p>
      {/if}
      <p></p>
    </div>
  </main>
{:else}
  <main class="page flex flex-col p-2 space-y-3">
    <div class="flow-root space-x-3">
      <select bind:value={state} class="select select-sm select-bordered">
        <option value="search">Search</option>
        <option value="capture">Capture</option>
      </select>
      <div class="float-right flex items-center space-x-3">
        <DatabaseSelector bind:loading />
      </div>
    </div>
    <hr />
    {#if state == "search"}
      <Search />
    {:else if state == "capture"}
      <Capture />
    {/if}
  </main>
{/if}

<style>
  .page {
    @apply absolute top-0 left-0 w-screen min-h-screen overflow-hidden;
  }
</style>
