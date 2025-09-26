<script lang="ts">
  import {
    database,
    openAboutWindow,
    type BuiltinDatabaseSources,
  } from "$lib/bindings";
  import { Channel } from "@tauri-apps/api/core";

  let name: BuiltinDatabaseSources = $state("dbip");
  let progress: number | null = $state(null);
  let stage: string | null = $state(null);

  const download = async () => {
    await database.download(
      name,
      new Channel((s) => (stage = s)),
      new Channel((p) => (progress = p)),
    );

    stage = null;
    progress = null;
  };

  $inspect(database.loading);
</script>

<main class="flex h-screen items-center justify-center select-none">
  <button class="btn absolute top-2 left-2" onclick={openAboutWindow}>
    ?
  </button>

  <div class="flex max-w-120 flex-col space-y-3 p-5 text-center">
    <h1 class="text-2xl font-semibold">Welcome to Ipmap</h1>
    <p>Download a database:</p>
    <div class="join">
      <select
        bind:value={name}
        disabled={database.loading}
        class="select select-sm join-item w-56"
      >
        <option value="dbip">DB-IP</option>
        <option value="geolite2">GeoLite2</option>
      </select>
      <button
        class="btn btn-sm join-item"
        disabled={database.loading}
        onclick={download}
      >
        Download
      </button>
    </div>

    {#if database.loading}
      {#if progress}
        <div class="mx-auto flex items-center space-x-2 text-sm">
          <progress class="progress w-56" value={progress} max="1"></progress>
          <p>{(progress * 100).toFixed(0)}%</p>
        </div>
        {#if stage}
          <p class="text-sm">{stage}...</p>
        {/if}
      {:else}
        <p>Previous loading session...</p>
      {/if}
    {/if}
  </div>
</main>
