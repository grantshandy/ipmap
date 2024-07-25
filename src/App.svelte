<script lang="ts">
  import Search from "./components/SearchMode.svelte";
  import Capture from "./components/CaptureMode.svelte";
  import Reverse from "./components/ReverseMode.svelte";
  import Traceroute from "./components/TracerouteMode.svelte";
  import DatabaseSelector from "./components/DatabaseSelector.svelte";

  import InfoIcon from "./assets/info-icon.svg?raw";

  import { fade } from "svelte/transition";
  import { onMount } from "svelte";
  import { openAboutWindow, traceroute } from "./bindings";
  import { theme } from "./utils/theme";
  import { database } from "./utils/database";
  import { platform } from "@tauri-apps/plugin-os";
  import Link from "./components/Link.svelte";

  let view: "search" | "capture" | "reverse" | "traceroute" =
    localStorage.view ?? "search";
  $: localStorage.view = view;

  onMount(() => {
    database.startLoading("Internal Databases");
    database.updateListings();
  });
</script>

<div class="relative h-screen w-screen overflow-hidden" data-theme={$theme}>
  {#if $database.ipv4dbs.length != 0 || $database.ipv6dbs.length != 0}
    <main
      transition:fade={{ duration: 100 }}
      class="page flex flex-col space-y-3 p-2"
    >
      <div class="flex items-center space-x-3">
        <div
          class="tooltip tooltip-right"
          data-tip={((view) => {
            if (view == "search") return "View an IP's location";
            if (view == "reverse")
              return "Find IP blocks nearest to the marker";
            if (view == "capture") return "View connected peers in real time";
            if (view == "traceroute")
              return "View the path data takes to its destination";
          })(view)}
        >
          <select bind:value={view} class="select select-bordered select-sm">
            <option value="search">Search</option>
            <option value="reverse">Reverse Search</option>
            <option value="capture">Capture</option>
            <option value="traceroute">Traceroute</option>
          </select>
        </div>
        <div class="flex grow items-center justify-end space-x-3">
          <DatabaseSelector />
          <button
            on:click={() => openAboutWindow($theme)}
            class="btn btn-square btn-primary btn-sm"
          >
            {@html InfoIcon}
          </button>
        </div>
      </div>
      {#if view == "search"}
        <Search />
      {:else if view == "reverse"}
        <Reverse />
      {:else}
        {#await traceroute.isPrivileged() then privileged}
          <!-- capture needs privileges on non-win32 systems -->
          {#if view == "capture" && ((platform() != "windows" && privileged) || platform() == "windows")}
            <Capture />

            <!-- traceroute needs privileges on all systems -->
          {:else if view == "traceroute" && privileged}
            <Traceroute />

            <!-- don't have the required privileges for the current mode, tell the current user -->
          {:else}
            <div class="flex grow items-center justify-center">
              <h1 class="text-lg font-semibold">
                {#if platform() == "windows"}
                  Run in Administrator mode to enable this feature.
                {:else}
                  <span class="code">CAP_NET_RAW</span> or root privileges required
                  for this mode.
                {/if}
              </h1>
            </div>
          {/if}
        {/await}
      {/if}
    </main>
  {:else}
    <main
      transition:fade={{ duration: 100 }}
      class="page flex items-center justify-center"
    >
      <div class="absolute right-5 top-5">
        <button
          on:click={() => openAboutWindow($theme)}
          class="btn btn-square btn-primary btn-sm"
        >
          {@html InfoIcon}
        </button>
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
            <Link href="https://github.com/sapics/ip-location-db">
              ip-location-db
            </Link> repository.
          </p>
        {:else}
          <p class="text-xl italic">Loading {$database.loading}...</p>
          <span class="loading loading-spinner loading-lg"></span>
        {/if}
      </div>
    </main>
  {/if}
</div>
