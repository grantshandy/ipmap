<script lang="ts">
  import { fade } from "svelte/transition";

  import Capture from "./Capture.svelte";
  import DatabaseSelector from "./DatabaseSelector.svelte";
  import Search from "./Search.svelte";
  import Reverse from "./Reverse.svelte";
  import ThemeSwitcher from "./ThemeSwitcher.svelte";

  let view: "search" | "capture" | "reverse" = localStorage.view ?? "capture";
  $: localStorage.view = view;
</script>

<main
  transition:fade={{ duration: 200 }}
  class="page flex flex-col space-y-3 p-2"
>
  <div class="flex items-center space-x-3">
    <select bind:value={view} class="select select-bordered select-sm">
      <option value="search">Search</option>
      <option value="capture">Capture</option>
      <option value="reverse">Reverse Search</option>
    </select>
    <ThemeSwitcher size={"1.5rem"} />
    <div class="flex grow items-center justify-end space-x-3">
      <DatabaseSelector />
    </div>
  </div>
  <hr />
  {#if view == "search"}
    <Search />
  {:else if view == "capture"}
    <Capture />
  {:else if view == "reverse"}
    <Reverse />
  {/if}
</main>
