<script lang="ts">
  import Link from "$lib/components/Link.svelte";
  import Databases from "$lib/components/Databases.svelte";
  import ErrorScreen from "$lib/components/ErrorScreen.svelte";

  import Search from "$lib/pages/Search.svelte";
  import Traceroute from "$lib/pages/Traceroute.svelte";
  import Capture from "$lib/pages/Capture.svelte";

  import { database, Pcap, openAboutWindow } from "$lib/bindings";
  import { basename } from "@tauri-apps/api/path";

  type Page = "capture" | "search" | "trace";

  let page: Page = $state((localStorage.page as Page) ?? "search");
  $effect(() => {
    localStorage.page = page;
  });

  let pcapResult = Pcap.init();
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
        onclick={database.openFile}
        disabled={database.loading != null}
        class="btn btn-primary mt-4"
      >
        {#if database.loading}
          {#if database.loading.progress == null}
            <span class="loading loading-spinner loading-xs"></span>
          {/if}

          Loading
          {#if database.loading.name}
            {database.loading.name}
          {/if}
          ...

          {#if database.loading.progress != null}
            <progress class="progress w-56" value={40} max={100}></progress>
          {/if}
        {:else}
          Open Database File
        {/if}
      </button>
    </div>
  </main>
{/snippet}
