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

  let state: "search" | "capture" | "reverse" = localStorage.page ?? "capture";
  $: localStorage.page = state;

  let loading: string | null = localStorage.loading ?? "Internal Database";
  $: localStorage.loading = loading;
</script>

{#if !$database}
  <!-- Intro page, prompts user to input database -->
  <main
    transition:fade={{ duration: 200 }}
    class="page flex items-center justify-center"
    data-theme={$theme}
  >
    <div class="select-none space-y-9 text-center">
      {#if !loading}
        <h1 class="text-2xl font-bold">Load an IP-Geolocation Database</h1>
      {/if}
      <DatabaseSelector bind:loading />
      {#if !loading}
        <p class="mx-auto max-w-sm leading-loose">
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
      <div class="absolute left-5 top-5">
        <ThemeSwitcher size={"1.25rem"} />
      </div>
    {/if}
  </main>
{:else}
  <!-- Main page with switcher for different modes -->
  <main class="page flex h-screen flex-col space-y-3 p-2" data-theme={$theme}>
    <div class="flex items-center space-x-3">
      <select bind:value={state} class="select select-bordered select-sm">
        <option value="search">Search</option>
        <option value="capture">Capture</option>
        <option value="reverse">Reverse Search</option>
      </select>
      <ThemeSwitcher size={"1.5rem"} />
      <div class="flex grow items-center justify-end space-x-3">
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
    @apply absolute left-0 top-0 min-h-screen w-screen overflow-hidden;
  }
</style>
