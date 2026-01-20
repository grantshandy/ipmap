<script lang="ts">
  import Search from "$lib/pages/Search.svelte";
  import Traceroute from "$lib/pages/Traceroute.svelte";
  import Capture from "$lib/pages/Capture.svelte";

  import AnimatedProgress from "$lib/components/AnimatedProgress.svelte";
  import Link from "$lib/components/Link.svelte";
  import ErrorScreen from "$lib/components/ErrorScreen.svelte";

  import { openAboutWindow } from "tauri-plugin-ipmap-api";
  import database, { type DatabaseSource } from "tauri-plugin-ipgeo-api";
  import { Pcap } from "tauri-plugin-pcap-api";

  import { pageState } from "$lib/page.svelte";

  let pcapResult = Pcap.init();
  let downloadSource: DatabaseSource = $state("dbipcombined");
</script>

{#if !database.responseBack}
  {@render loading()}
{:else if !database.anyEnabled}
  {@render welcome()}
{:else}
  {@render main()}
{/if}

{#snippet loading()}
  <div
    class="flex h-screen max-h-screen flex-col items-center justify-center space-y-4"
  >
    <span class="loading loading-spinner loading-xl"></span>
    <p class="italic">Loading Databases...</p>
  </div>
{/snippet}

{#snippet main()}
  <main class="flex h-screen max-h-screen flex-col overscroll-none">
    {#if pageState.page === "search"}
      <Search />
    {:else if pageState.page === "trace"}
      <Traceroute />
    {:else if pageState.page === "capture"}
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
        <div class="mt-4 flex flex-col items-center space-y-3">
          <p class="italic">
            Loading
            {database.loading.name ?? ""}
            ...
          </p>

          {#if database.loading.progress != null}
            <AnimatedProgress value={database.loading.progress} class="w-64" />
          {:else}
            <span class="loading loading-spinner"></span>
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
