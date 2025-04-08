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

  let info: {
    ip: string;
    coord: Coordinate | null;
    loc: Location | null;
  } | null = $state(null);

  const search = async (ev: Event) => {
    ev.preventDefault();

    const resp = await commands.lookupIp(ip);
    info = { ip, coord: resp ? resp[0] : null, loc: resp ? resp[1] : null };
  };
</script>

<main class="min-h-screen p-3 space-y-3">
  <Databases bind:appState />

  {#if appState.ipv4.selected || appState.ipv6.selected}
    <form onsubmit={search}>
      <input class="input" type="text" bind:value={ip} />
      <button class="btn btn-primary">Search</button>
    </form>

    {#if info}
      <div class="flex flex-col gap-2">
        <h2 class="text-2xl font-bold">Info</h2>
        {#if info.coord}
          <p>({info.coord.lat},{info.coord.lng})</p>
        {/if}
        {#if info.loc}
          <p>{info.loc.city}, {info.loc.region}, {info.loc.country_code}</p>
        {/if}
        {#if !info.loc && !info.coord}
          <p class="text-red-500">No location found</p>
        {/if}
      </div>
    {/if}
  {/if}
</main>
