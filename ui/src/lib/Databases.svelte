<script lang="ts">
    import { database, type DbCollectionInfo } from "../bindings";
    import { basename } from "@tauri-apps/api/path";
    import ArrowIntoBoxIcon from "$lib/../assets/arrow-into-box-symbolic.svg?raw";
    import UserTrashIcon from "$lib/../assets/user-trash-symbolic.svg?raw";
</script>

<div class="float-right flex justify-end space-x-2 select-none items-center">
    {#if database.ipv4.loaded.length > 0}
        {@render databaseSelector(database.ipv4, false)}
    {/if}

    {#if database.ipv6.loaded.length > 0}
        {@render databaseSelector(database.ipv6, true)}
    {/if}

    <button
        onclick={database.open}
        class="btn btn-sm btn-primary float-right"
        disabled={database.loading != null}
    >
        {#if database.loading}
            <span class="loading loading-spinner loading-xs"></span>
            Loading...
        {:else}
            {@html ArrowIntoBoxIcon}
        {/if}
    </button>
</div>

{#snippet databaseSelector(dbs: DbCollectionInfo, ipv6: boolean)}
    <div
        class="items-center join join-horizontal rounded-md max-w-72"
        class:bg-base-200={dbs.loaded.length === 1}
        class:bg-base-300={dbs.loaded.length > 1}
    >
        <span class="text-xs pl-2.5 pr-1">IPv{ipv6 ? "6" : "4"}: </span>
        <select
            class="select select-sm select-ghost join-item grow"
            disabled={dbs.loaded.length < 2}
            onchange={(ev) => database.setSelected(ev.currentTarget.value)}
        >
            {#each dbs.loaded as name}
                <option value={name} selected={name === dbs.selected}>
                    {#await basename(name) then filename}
                        {filename || name}
                    {/await}
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
