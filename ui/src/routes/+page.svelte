<script lang="ts">
  import Link from "$lib/components/Link.svelte";
  import Databases from "$lib/components/Databases.svelte";
  import ErrorScreen from "$lib/components/ErrorScreen.svelte";

  import Search from "$lib/pages/Search.svelte";
  import Traceroute from "$lib/pages/Traceroute.svelte";
  import Capture from "$lib/pages/Capture.svelte";

  import { openAboutWindow, Pcap } from "$lib/bindings";
  import database, { type DatabaseSource } from "tauri-plugin-ipgeo-api";

  type Page = "capture" | "search" | "trace";

  let page: Page = $state((localStorage.page as Page) ?? "search");
  $effect(() => {
    localStorage.page = page;
  });

  let pcapResult = Pcap.init();

  let downloadSource: DatabaseSource = $state("dbipcombined");
</script>

{#if !database.anyEnabled}
  {@render welcome()}
{:else}
  {@render main()}
{/if}

{#snippet main()}
  <main class="flex h-screen max-h-screen flex-col overscroll-none">
    <div class="flow-root w-full p-3 select-none">
      <select class="select select-sm max-w-40" bind:value={page}>
        <option value="search">Location Search</option>
        <option value="capture">Monitor Network</option>
        <option value="trace">Traceroute</option>
      </select>

      <button class="btn btn-sm" onclick={openAboutWindow}>?</button>

      <Databases />
    </div>

    {#if page === "search"}
      <Search />
    {:else if page === "trace"}
      <Traceroute />
    {:else if page === "capture"}
      {#await pcapResult then result}
        {#if result.status == "ok"}
          <Capture pcap={result.data} />
        {:else}
          <ErrorScreen error={result.error} />
        {/if}
      {/await}
    {/if}
  </main>
{/snippet}

{#snippet welcome()}
  <main class="flex h-screen items-center justify-center select-none">
    <button class="btn absolute top-2 left-2" onclick={openAboutWindow}>
      ?
    </button>

    <div class="max-w-120 space-y-12 p-5 text-center">
      <h1 class="text-3xl font-semibold">Welcome to Ipmap</h1>

      <div class="space-y-3">
        <div class="join join-horizontal">
          <select
            bind:value={downloadSource}
            disabled={database.loading != null}
            class="select join-item"
          >
            <option value={"dbipcombined"}>DB-IP</option>
            <option value={"geolite2combined"}>GeoLite2</option>
          </select>
          <button
            class="btn btn-primary join-item"
            disabled={database.loading != null}
            onclick={() => database.downloadSource(downloadSource)}
            >Download Ip Geolocation Database</button
          >
        </div>
        <p>or</p>
        <button
          onclick={database.openFile}
          disabled={database.loading != null}
          class="btn"
        >
          Open Database File
        </button>
      </div>

      {#if database.loading != null}
        <div class="mt-4">
          {#if database.loading.progress == null}
            <span class="loading loading-spinner loading-xs"></span>
          {/if}

          <p class="italic">
            Loading
            {#if database.loading.name}
              {database.loading.name}
            {/if}
            ...
          </p>

          {#if database.loading.progress != null}
            <progress
              class="progress w-64"
              value={database.loading.progress * 100}
              max={100}
            ></progress>
          {/if}
        </div>
      {:else}
        <div class="space-y-2">
          <p>
            Load an ip-geolocation database to get started. It supports loading
            any file in this format:
          </p>
          <p>
            <code class="bg-base-200 rounded-sm p-1 shadow-md select-text">
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
        </div>
      {/if}
    </div>
  </main>
{/snippet}
