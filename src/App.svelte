<script lang="ts">
  import { fade } from "svelte/transition";
  import { traceroute } from "./bindings";
  import { theme } from "./utils/theme";
  import { database } from "./utils/database";
  import { open } from "@tauri-apps/plugin-shell";
  import { platform } from "@tauri-apps/plugin-os";

  import Search from "./modes/Search.svelte";
  import Capture from "./modes/Capture.svelte";
  import Reverse from "./modes/Reverse.svelte";
  import Traceroute from "./modes/Traceroute.svelte";

  import ThemeSwitcher from "./components/ThemeSwitcher.svelte";
  import DatabaseSelector from "./components/DatabaseSelector.svelte";
  import InfoWindowButton from "./components/AboutWindowButton.svelte";

  let view: "search" | "capture" | "reverse" | "traceroute" =
    localStorage.view ?? "search";
  $: localStorage.view = view;

  database.startLoading("Internal Databases");
  database.updateListings();
</script>

<div
  class="relative h-screen w-screen overflow-hidden"
  data-theme={$theme.isLight ? $theme.light : $theme.dark}
>
  {#if $database.ipv4dbs.length != 0 || $database.ipv6dbs.length != 0}
    <main
      transition:fade={{ duration: 100 }}
      class="page flex flex-col space-y-3 p-2"
    >
      <div class="flex items-center space-x-3">
        <select bind:value={view} class="select select-bordered select-sm">
          <option value="search">Search</option>
          <option value="reverse">Reverse Search</option>
          <option value="capture">Capture</option>
          <option value="traceroute">Traceroute</option>
        </select>
        <ThemeSwitcher />
        <InfoWindowButton />
        <div class="flex grow items-center justify-end space-x-3">
          <DatabaseSelector />
        </div>
      </div>
      {#if view == "search"}
        <Search />
      {:else if view == "reverse"}
        <Reverse />
      {:else}
        {#await traceroute.isPrivileged() then privileged}
          {#await platform() then platform}
            <!-- capture needs privileges on non-win32 systems -->
            {#if view == "capture" && ((platform != "windows" && privileged) || platform == "windows")}
              <Capture />

              <!-- traceroute needs privileges on all systems -->
            {:else if view == "traceroute" && privileged}
              <Traceroute />

              <!-- don't have the required privileges for the current mode -->
            {:else}
              <div class="flex grow items-center justify-center">
                {#if platform == "windows"}
                  <h1 class="text-lg font-semibold">
                    Run in Administrator mode to enable this feature.
                  </h1>
                {:else}
                  <h1 class="text-lg font-semibold">
                    <span class="code">CAP_NET_RAW</span> or root privileges required
                    for this mode.
                  </h1>
                {/if}
              </div>
            {/if}
          {/await}
        {/await}
      {/if}
    </main>
  {:else}
    <main
      transition:fade={{ duration: 100 }}
      class="page flex items-center justify-center"
    >
      <div class="absolute left-5 top-5 flex space-x-3">
        <ThemeSwitcher />
        <InfoWindowButton />
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
