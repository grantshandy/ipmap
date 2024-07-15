<script lang="ts">
  import ThemeSwitcher from "./ThemeSwitcher.svelte";

  import { open } from "@tauri-apps/api/shell";
  import { fade } from "svelte/transition";
  import { database } from "../stores/database";
</script>

<main
  transition:fade={{ duration: 200 }}
  class="page flex items-center justify-center"
>
  <div class="absolute left-5 top-5">
    <ThemeSwitcher size={"1.25rem"} />
  </div>
  <div class="select-none space-y-9 text-center">
    <h1 class="text-2xl font-bold">Load an IP-Geolocation Database</h1>

    {#if !$database.loading}
      <button class="btn btn-primary" on:click={database.importDatabase}
        >Load Database</button
      >
    {:else}
      <p class="text-xl italic">
        Loading {$database.loading}...
        <span class="loading-xl loading loading-spinner"></span>
      </p>
    {/if}
    <p class="mx-auto max-w-sm leading-loose">
      Databases must be in the <span class="code">*-city-ipvX-num.csv</span>
      format, and can be found at the
      <button
        on:click={() => open("https://github.com/sapics/ip-location-db")}
        class="text-success underline">ip-location-db</button
      > repository.
    </p>
  </div>
</main>
