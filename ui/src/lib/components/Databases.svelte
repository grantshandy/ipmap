<script lang="ts">
  import ArrowIntoBoxIcon from "$lib/../assets/arrow-into-box-symbolic.svg?raw";
  import UserTrashIcon from "$lib/../assets/user-trash-symbolic.svg?raw";

  import { database, type DbSetInfo } from "$lib/bindings";
</script>

<div class="float-right flex items-center justify-end space-x-2 select-none">
  {#if database.ipv4Enabled && database.ipv4.selected != null}
    {@render databaseSelector(database.ipv4, "IPv4")}
  {/if}

  {#if database.ipv6Enabled && database.ipv6.selected != null}
    {@render databaseSelector(database.ipv4, "IPv6")}
  {/if}

  {#if database.combinedEnabled}
    {@render databaseSelector(database.combined, "Combined")}
  {/if}

  <button
    onclick={database.openFile}
    class="btn btn-sm btn-primary float-right"
    disabled={database.loading != null}
  >
    {#if database.loading}
      <span class="loading loading-spinner loading-xs"></span>
    {:else}
      {@html ArrowIntoBoxIcon}
    {/if}
  </button>
</div>

{#snippet databaseSelector(dbs: DbSetInfo, label: string)}
  <div
    class="join join-horizontal max-w-72 items-center rounded-md"
    class:bg-base-200={dbs.loaded.length === 1}
    class:bg-base-300={dbs.loaded.length > 1}
  >
    <span class="pr-1 pl-2.5 text-xs">{label}: </span>
    <select
      class="select select-sm select-ghost join-item grow"
      disabled={dbs.loaded.length < 2}
      onchange={(ev) => database.setSelected(ev.currentTarget.value)}
    >
      {#each dbs.loaded as name}
        <option value={name} selected={name === dbs.selected}>
          {name}
        </option>
      {/each}
    </select>
    <button
      disabled={dbs.selected == null}
      onclick={() => database.unload(dbs.selected)}
      class="btn btn-sm btn-error join-item"
      aria-label="Unload Database"
    >
      {@html UserTrashIcon}
    </button>
  </div>
{/snippet}
