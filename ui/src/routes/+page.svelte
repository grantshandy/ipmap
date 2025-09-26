<script lang="ts">
  import Link from "$lib/components/Link.svelte";
  import Databases from "$lib/components/Databases.svelte";
  // import ErrorScreen from "$lib/components/ErrorScreen.svelte";

  import Search from "$lib/pages/Search.svelte";
  // import Traceroute from "$lib/pages/Traceroute.svelte";
  // import Capture from "$lib/pages/Capture.svelte";

  import {
    database,
    // Pcap,
    openAboutWindow,
    type BuiltinDatabaseSources,
  } from "$lib/bindings";
  import { basename } from "@tauri-apps/api/path";
  import Welcome from "$lib/pages/Welcome.svelte";

  type Page = "capture" | "search" | "trace";

  let page: Page = $state((localStorage.page as Page) ?? "search");
  $effect(() => {
    localStorage.page = page;
  });

  // let pcapResult = Pcap.init();
</script>

{#await database.initCache()}
  {@render loading()}
{:then}
  {#if !database.anyEnabled}
    <Welcome />
  {:else}
    {@render main()}
  {/if}
{/await}

{#snippet main()}
  <main class="flex h-screen max-h-screen flex-col overscroll-none">
    <div class="flex w-full p-3 select-none">
      <select class="select select-sm max-w-40" bind:value={page}>
        <option value="search">Location Search</option>
        <!-- <option value="capture">Monitor Network</option> -->
        <!-- <option value="trace">Traceroute</option> -->
      </select>

      <button class="btn btn-sm" onclick={openAboutWindow}>?</button>
    </div>

    {#if page === "search"}
      <Search />
      <!-- {:else if page === "trace"}
      <Traceroute />
    {:else if page === "capture"}
      {#await pcapResult then result}
        {#if result.status == "ok"}
          <Capture pcap={result.data} />
        {:else}
          <ErrorScreen error={result.error} />
        {/if}
      {/await} -->
    {/if}
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
