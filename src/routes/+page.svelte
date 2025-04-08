<script lang="ts">
  import {
    commands,
    type AppStateInfo,
    type Coordinate,
    type Location,
  } from "../bindings";
  import Databases from "../lib/Databases.svelte";

  let appState: AppStateInfo = $state({
    ipv4: { loaded: [], selected: null },
    ipv6: { loaded: [], selected: null },
    loading: null,
  });

  let ip = $state("");
  let location: [Coordinate, Location] | null = $state(null);

  const search = async (ev: Event) => {
    ev.preventDefault();

    const resp = await commands.lookupIp(ip);
    console.log(resp);
  };
</script>

<main class="w-full min-h-screen p-3 space-y-3">
  <h1 class="text-3xl font-bold select-none">Ipmap</h1>

  <Databases bind:appState />

  {#if appState.ipv4.selected || appState.ipv6.selected}
    <form onsubmit={search}>
      <input class="input" type="text" bind:value={ip} />
      <button class="btn btn-primary">Search</button>
    </form>

    {#if location}
      <div class="flex flex-col gap-2">
        <h2 class="text-2xl font-bold">Location Info</h2>
        <pre>{JSON.stringify(location, null, 2)}</pre>
      </div>
    {/if}
  {/if}
</main>
