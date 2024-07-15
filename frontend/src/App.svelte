<script lang="ts">
  import IntroPage from "./components/IntroPage.svelte";
  import MainPage from "./components/MainPage.svelte";

  import { theme } from "./stores/theme";
  import { database } from "./stores/database";
</script>

<main class="relative h-screen w-screen overflow-hidden" data-theme={$theme}>
  {#if $database.ipv4dbs.length != 0 || $database.ipv6dbs.length != 0}
    <MainPage />
  {:else}
    <IntroPage />
  {/if}
</main>
