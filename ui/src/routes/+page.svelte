<script lang="ts">
  import Search from "$lib/Search.svelte";
  import Databases from "$lib/Databases.svelte";
  import Capture from "$lib/capture/Page.svelte";
  import Traceroute from "$lib/traceroute/Page.svelte";

  import { database, newPcapInstance, type Pcap } from "../bindings";
  import { openUrl } from "@tauri-apps/plugin-opener";

  const openDownload = () =>
    openUrl("https://github.com/sapics/ip-location-db?tab=readme-ov-file#city");

  type Page = "capture" | "search" | "trace";

  // not super robust, but this works for reloading the page in development :)
  let page: Page = $state((localStorage.page as Page) ?? "search");
  $effect(() => {
    localStorage.page = page;
  });
</script>

<main class="flex h-screen flex-col space-y-3 overscroll-none p-3">
  {#if !database.anyEnabled}
    {@render welcomePage()}
  {:else}
    {@render mainPage()}
  {/if}
</main>

{#snippet welcomePage()}
  <div class="space-y-3">
    <h1 class="text-2xl font-semibold">Ipmap</h1>
    <p>
      Load a ip-geolocation database to get started. <button
        class="link"
        onclick={openDownload}>Download link</button
      >
    </p>
    <button
      onclick={database.open}
      disabled={database.loading != null}
      class="btn btn-primary"
    >
      {#if database.loading}
        <span class="loading loading-spinner loading-xs"></span>
        Loading...
      {:else}
        Open Database File
      {/if}
    </button>
  </div>
{/snippet}

{#snippet mainPage()}
  <div class="flow-root w-full select-none">
    <select class="select select-sm max-w-40" bind:value={page}>
      <option value="capture">Packet Capture</option>
      <option value="search">Location Search</option>
      <option value="trace">Traceroute</option>
    </select>
    <Databases />
  </div>

  {#if page === "search"}
    <Search />
  {:else if page === "trace"}
    <Traceroute />
  {:else if page === "capture"}
    <Capture />
  {/if}
{/snippet}
