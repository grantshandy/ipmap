<script lang="ts">
  import { database } from "$lib/bindings";

  import Main from "$lib/Main.svelte";
  import Welcome from "$lib/Welcome.svelte";
</script>

{#if !database.anyEnabled}
  <Welcome />
{:else}
  <Main />
{/if}
