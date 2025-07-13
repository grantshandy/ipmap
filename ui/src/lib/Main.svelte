<script lang="ts">
  import Search from "$lib/pages/Search.svelte";
  import Capture from "$lib/pages/Capture.svelte";
  import Traceroute from "$lib/pages/Traceroute.svelte";

  import Databases from "$lib/components/Databases.svelte";
  import ErrorScreen from "$lib/components/ErrorScreen.svelte";

  import { newPcapInstance, utils } from "$lib/bindings";

  type Page = "capture" | "search" | "trace";

  let page: Page = $state((localStorage.page as Page) ?? "search");
  $effect(() => {
    localStorage.page = page;
  });
</script>

<main class="flex h-screen flex-col space-y-3 overscroll-none p-3">
  <div class="flow-root w-full select-none">
    <select class="select select-sm max-w-40" bind:value={page}>
      <option value="search">Location Search</option>
      <option value="capture">Packet Capture</option>
      <option value="trace">Traceroute</option>
    </select>

    <button class="btn btn-sm" onclick={utils.openAboutWindow}>?</button>

    <Databases />
  </div>

  {#if page === "search"}
    <Search />
  {:else if page === "trace"}
    <Traceroute />
  {:else if page === "capture"}
    {#await newPcapInstance() then result}
      {#if result.status == "ok"}
        <Capture pcap={result.data} />
      {:else}
        <ErrorScreen error={result.error} />
      {/if}
    {/await}
  {/if}
</main>
