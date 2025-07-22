<script lang="ts">
  import Link from "$lib/components/Link.svelte";
  import Databases from "$lib/components/Databases.svelte";
  import ErrorScreen from "$lib/components/ErrorScreen.svelte";

  import Search from "$lib/pages/Search.svelte";
  import Traceroute from "$lib/pages/Traceroute.svelte";
  import Capture from "$lib/pages/Capture.svelte";
  import GlobeCapture from "$lib/pages/GlobeCapture.svelte";

  import { database, Pcap, utils } from "$lib/bindings";
  import { basename } from "@tauri-apps/api/path";

  type Page = "capture" | "search" | "trace" | "globe";

  let page: Page = $state((localStorage.page as Page) ?? "search");
  $effect(() => {
    localStorage.page = page;
  });

  let pcapResult = Pcap.init();
</script>

{#await database.loadInternals()}
  {@render loading()}
{:then}
  {#if !database.anyEnabled}
    {@render welcome()}
  {:else}
    {@render main()}
  {/if}
{/await}

{#snippet main()}
  <main class="flex h-screen flex-col overscroll-none">
    <div class="flow-root w-full p-3 select-none">
      <select class="select select-sm max-w-40" bind:value={page}>
        <option value="search">Location Search</option>
        <option value="capture">Monitor Network</option>
        <option value="globe">Network Globe</option>
        <option value="trace">Traceroute</option>
      </select>

      <button class="btn btn-sm" onclick={utils.openAboutWindow}>?</button>

      <Databases />
    </div>

    {#if page === "search"}
      <Search />
    {:else if page === "trace"}
      <Traceroute />
    {:else}
      {#await pcapResult then result}
        {#if result.status == "ok"}
          {#if page === "capture"}
            <Capture pcap={result.data} />
          {:else if page === "globe"}
            <GlobeCapture pcap={result.data} />
          {/if}
        {:else}
          <ErrorScreen error={result.error} />
        {/if}
      {/await}
    {/if}
  </main>
{/snippet}

{#snippet welcome()}
  <main class="flex h-screen items-center justify-center select-none">
    <div class="max-w-120 space-y-3 p-5 text-center">
      <h1 class="text-2xl font-semibold">Welcome to Ipmap</h1>
      <p>
        Load an ip-geolocation database to get started. It supports loading any
        file in this format:
      </p>
      <p>
        <code class="bg-base-200 rounded-sm p-1">
          [dbip/geolite2]-city-[ipv4/ipv6].csv[.gz]
        </code>
      </p>
      <p>
        You can download them
        <Link
          href="https://github.com/sapics/ip-location-db?tab=readme-ov-file#city"
          >here</Link
        >.
      </p>
      <button
        onclick={database.open}
        disabled={database.loading != null}
        class="btn btn-primary mt-4"
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

{#snippet loading()}
  <main
    class="flex h-screen w-screen flex-col items-center justify-center space-y-3"
  >
    <span class="loading loading-spinner loading-xl"></span>
    <p class="text-lg">Initializing Internal Databases</p>
  </main>
{/snippet}
