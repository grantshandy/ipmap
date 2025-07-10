<script lang="ts">
  import { database } from "$lib/bindings";

  import Main from "$lib/Main.svelte";
  import Welcome from "$lib/Welcome.svelte";
</script>

{#await database.loadInternals()}
  <div
    class="flex h-screen w-screen flex-col items-center justify-center space-y-3"
  >
    <span class="loading loading-spinner loading-xl"></span>
    <p class="text-lg">Initializing Internal Databases</p>
  </div>
{:then}
  {#if !database.anyEnabled}
    <Welcome />
  {:else}
    <Main />
  {/if}
{/await}
