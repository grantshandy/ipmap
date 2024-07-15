<script lang="ts">
  import { fade } from "svelte/transition";
  import { theme } from "./stores/theme";
  import { database } from "./stores/database";
  import { open } from "@tauri-apps/api/shell";

  import ThemeSwitcher from "./components/ThemeSwitcher.svelte";
  import DatabaseSelector from "./components/DatabaseSelector.svelte";
  import Search from "./components/Search.svelte";
  import Capture from "./components/Capture.svelte";
  import Reverse from "./components/Reverse.svelte";
  import Traceroute from "./components/Traceroute.svelte";

  let view: "search" | "capture" | "reverse" | "traceroute" = localStorage.view ?? "capture";
  $: localStorage.view = view;
</script>

<div class="relative h-screen w-screen overflow-hidden" data-theme={$theme}>
  {#if $database.ipv4dbs.length != 0 || $database.ipv6dbs.length != 0}
    <main
      transition:fade={{ duration: 200 }}
      class="page flex flex-col space-y-3 p-2"
    >
      <div class="flex items-center space-x-3">
        <select bind:value={view} class="select select-bordered select-sm">
          <option value="search">Search</option>
          <option value="capture">Capture</option>
          <option value="reverse">Reverse Search</option>
          <option value="traceroute">Traceroute</option>
        </select>
        <ThemeSwitcher size={"1.5rem"} />
        <div class="flex grow items-center justify-end space-x-3">
          <DatabaseSelector />
        </div>
      </div>
      <hr />
      {#if view == "search"}
        <Search />
      {:else if view == "capture"}
        <Capture />
      {:else if view == "reverse"}
        <Reverse />
      {:else if view == "traceroute"}
        <Traceroute />
      {/if}
    </main>
  {:else}
    <main
      transition:fade={{ duration: 200 }}
      class="page flex items-center justify-center"
    >
      <div class="absolute left-5 top-5">
        <ThemeSwitcher size={"1.25rem"} />
      </div>
      <div class="select-none space-y-9 text-center">
        {#if !$database.loading}
          <h1 class="text-2xl font-bold">Load an IP-Geolocation Database</h1>
          <button class="btn btn-primary" on:click={database.importDatabase}
            >Load Database</button
          >
          <p class="mx-auto max-w-sm leading-loose">
            Databases must be in the <span class="code"
              >*-city-ipvX-num.csv</span
            >
            format, and can be found at the
            <button
              on:click={() => open("https://github.com/sapics/ip-location-db")}
              class="text-success underline">ip-location-db</button
            > repository.
          </p>
        {:else}
          <p class="text-xl italic">Loading {$database.loading}...</p>
          <span class="loading loading-spinner loading-lg"></span>
        {/if}
      </div>
    </main>
  {/if}
</div>
