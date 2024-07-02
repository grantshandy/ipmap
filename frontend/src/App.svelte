<script lang="ts">
  import Map from "./lib/Map.svelte";
  import CaptureMenu from "./lib/CaptureMenu.svelte";
  import SearchMenu from "./lib/SearchMenu.svelte";
  import DatabaseSelector from "./lib/DatabaseSelector.svelte";

  import { map, ApplicationMode } from "./map";
  import { type DatabaseInfo } from "./bindings";

  let loading: string | null = null;
  let database: DatabaseInfo | null = null;

  let query: string = "";
  let state: ApplicationMode = ApplicationMode.CAPTURE;
  $: map.setMode(state);

  let loadingModal: HTMLDialogElement;
  $: if (loadingModal && loading) loadingModal.showModal();
  $: if (loadingModal && !loading) loadingModal.close();
</script>

<main class="p-4 space-y-4 w-screen h-screen flex flex-col">
  <div class="flex">
    <div class="grow inline-block">
      <DatabaseSelector bind:database bind:loading />
    </div>
    <div class="flex items-center space-x-2">
      <select class="select select-bordered select-sm" bind:value={state}>
        <option value={ApplicationMode.CAPTURE}>Capture</option>
        <option value={ApplicationMode.SEARCH}>Search</option>
      </select>

      <div class="inline-block">
        {#if state == ApplicationMode.CAPTURE}
          <CaptureMenu bind:database bind:loading />
        {:else}
          <SearchMenu bind:database bind:loading bind:query />
        {/if}
      </div>
    </div>
  </div>
  <Map bind:state bind:query />
</main>

<dialog class="modal" bind:this={loadingModal}>
  <div class="modal-box flow-root">
    <p class="float-left">Loading {loading}</p>
    <span class="float-right loading loading-spinner loading-md"></span>
  </div>
  <form method="dialog" class="modal-backdrop"></form>
</dialog>
