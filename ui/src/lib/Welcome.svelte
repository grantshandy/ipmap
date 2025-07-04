<script lang="ts">
  import Link from "./components/Link.svelte";

  import { database } from "$lib/bindings";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { basename } from "@tauri-apps/api/path";
</script>

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
