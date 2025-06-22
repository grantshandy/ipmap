<script lang="ts">
  import Search from "$lib/Search.svelte";
  import Capture from "$lib/Capture.svelte";
  import Databases from "$lib/Databases.svelte";

  import { database, newPcapInstance, type Pcap } from "../bindings";
  import { openUrl } from "@tauri-apps/plugin-opener";

  const openDownload = () =>
    openUrl("https://github.com/sapics/ip-location-db?tab=readme-ov-file#city");

  type Page = "capture" | "search";

  // not super robust, but this works for reloading the page in development :)
  let page: Page = $state((localStorage.page as Page) ?? "search");
  $effect(() => {
    localStorage.page = page;
  });
</script>

<main class="flex min-h-screen flex-col space-y-3 p-3">
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
      <option value={"capture"}>Packet Capture</option>
      <option value={"search"}>Location Search</option>
    </select>
    <Databases />
  </div>

  {#if page === "search"}
    <Search />
  {:else if page === "capture"}
    {#await newPcapInstance() then pcap}
      {#if typeof pcap == "string"}
        <div class="flex grow items-center justify-center">
          <div class="rounded-box bg-error max-w-96 space-y-2 px-3 py-2">
            <h1 class="text-lg font-semibold">
              Error Loading <code>libpcap</code>:
            </h1>
            <p class="text-sm">
              <code>{pcap}</code>
            </p>
          </div>
        </div>
      {:else if pcap != null}
        <Capture {pcap} />
      {/if}
    {/await}
  {/if}
{/snippet}
