<script lang="ts">
  import { open } from "@tauri-apps/api/shell";

  import { database } from "./stores/database";
  import { theme } from "./stores/theme";
  import { fade } from "svelte/transition";

  import Capture from "./components/Capture.svelte";
  import DatabaseSelector from "./components/DatabaseSelector.svelte";
  import Search from "./components/Search.svelte";
  import Reverse from "./components/Reverse.svelte";
  import ThemeSwitcher from "./components/ThemeSwitcher.svelte";

  let state: "search" | "capture" | "reverse" = localStorage.page ?? "reverse";
  $: localStorage.page = state;

  let loading: string | null = "Internal Database";
</script>

{#if !$database}
  <main
    transition:fade={{ duration: 200 }}
    class="page flex items-center justify-center"
    data-theme={$theme}
  >
    <div class="text-center space-y-9 select-none">
      {#if !loading}
        <h1 class="font-bold text-2xl">Load an IP-Geolocation Database</h1>
      {/if}
      <DatabaseSelector bind:loading />
      {#if !loading}
        <p class="max-w-sm mx-auto leading-loose">
          Databases must be in the <span class="code">*-city-ipv4-num.csv</span>
          format, and can be found at the
          <button
            on:click={() => open("https://github.com/sapics/ip-location-db")}
            class="text-success underline">ip-location-db</button
          > repository.
        </p>
      {/if}
    </div>
    {#if !loading}
      <div class="absolute top-5 left-5">
        <ThemeSwitcher size={"1.25rem"} />
      </div>
    {/if}
  </main>
{:else}
  <main class="page flex flex-col p-2 space-y-3 h-screen" data-theme={$theme}>
    <div class="flex space-x-3 items-center">
      <select bind:value={state} class="select select-sm select-bordered">
        <option value="search">Search</option>
        <option value="capture">Capture</option>
        <option value="reverse">Reverse Search</option>
      </select>
      <ThemeSwitcher size={"1.5rem"} />
      <div class="grow flex justify-end items-center space-x-3">
        <DatabaseSelector bind:loading />
      </div>
    </div>
    <hr />
    {#if state == "search"}
      <Search />
    {:else if state == "capture"}
      <Capture />
    {:else if state == "reverse"}
      <Reverse />
    {/if}
  </main>
{/if}

<style>
  .page {
    @apply absolute top-0 left-0 w-screen min-h-screen overflow-hidden;
  }
</style>
