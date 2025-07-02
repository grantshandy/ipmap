<script lang="ts">
  import Search from "$lib/Search.svelte";
  import Databases from "$lib/Databases.svelte";
  import Capture from "$lib/Capture.svelte";
  import Traceroute from "$lib/traceroute/Page.svelte";
  import ErrorScreen from "$lib/ErrorScreen.svelte";

  import { database, newPcapInstance } from "../bindings";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { basename } from "@tauri-apps/api/path";

  const openDownload = () =>
    openUrl("https://github.com/sapics/ip-location-db?tab=readme-ov-file#city");

  type Page = "capture" | "search" | "trace";

  let page: Page = $state((localStorage.page as Page) ?? "search");
  $effect(() => {
    localStorage.page = page;
  });
</script>

{#if !database.anyEnabled}
  {@render welcomePage()}
{:else}
  {@render mainPage()}
{/if}

{#snippet welcomePage()}
  <main class="flex h-screen items-center justify-center">
    <div
      class="bg-base-200 rounded-box border-base-300 max-w-120 space-y-3 border p-5 text-center shadow-xl"
    >
      <h1 class="text-2xl font-semibold">Welcome to Ipmap</h1>
      <p>
        Load an ip-geolocation database to get started. It supports loading any
        file in this format:
      </p>
      <p>
        <code class="bg-base-100 rounded-sm p-1">
          [dbip/geolite2]-city-[ipv4/ipv6].csv[.gz]
        </code>
      </p>
      <p>
        You can download them
        <button onclick={openDownload} class="link">here</button>.
      </p>
      <button
        onclick={database.open}
        disabled={database.loading != null}
        class="btn btn-primary"
      >
        {#if database.loading}
          <span class="loading loading-spinner loading-xs"></span>
          Loading
          {#await basename(database.loading) then filename}
            {filename ?? ""}...
          {/await}
        {:else}
          Open Database File
        {/if}
      </button>
    </div>
  </main>
{/snippet}

{#snippet mainPage()}
  <main class="flex h-screen flex-col space-y-3 overscroll-none p-3">
    <div class="flow-root w-full select-none">
      <select class="select select-sm max-w-40" bind:value={page}>
        <option value="search">Location Search</option>
        <option value="capture">Packet Capture</option>
        <option value="trace">Traceroute</option>
      </select>

      <Databases />
    </div>

    {#if page === "search"}
      <Search />
    {:else if page === "trace"}
      <Traceroute />
    {:else if page === "capture"}
      {#await newPcapInstance() then result}
        {#if result.status == "ok"}
          <Capture pcap={result.data} />
        {:else}
          <ErrorScreen error={result.error} />
        {/if}
      {/await}
    {/if}
  </main>
{/snippet}
