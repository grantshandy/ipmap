<script lang="ts">
    import { openUrl } from "@tauri-apps/plugin-opener";
    import { database, type DbCollectionInfo } from "../bindings";
    import { basename } from "@tauri-apps/api/path";

    const DB_DOWNLOAD_URL =
        "https://github.com/sapics/ip-location-db?tab=readme-ov-file#city";
</script>

<div
    class="flex flex-col space-y-2 w-100 select-none bg-base-200 border border-box border-base-300 rounded-box p-2"
>
    <div class="flex items-center space-x-4">
        <h2 class="font-semibold text-2xl">Databases</h2>
        <span class="grow"></span>
        <button
            class="link float-right"
            onclick={() => openUrl(DB_DOWNLOAD_URL)}>Download</button
        >
        <button
            onclick={database.open}
            class="btn btn-primary float-right"
            disabled={database.loading != null}
        >
            {#if database.loading}
                <span class="loading loading-spinner loading-xs"></span>
                Loading...
            {:else}
                Load
            {/if}
        </button>
    </div>

    {#if database.ipv4.loaded.length > 0}
        {@render databaseSelector(database.ipv4, false)}
    {/if}

    {#if database.ipv6.loaded.length > 0}
        {@render databaseSelector(database.ipv6, true)}
    {/if}
</div>

{#snippet databaseSelector(dbs: DbCollectionInfo, ipv6: boolean)}
    <div class="flex space-x-4 items-center grow">
        <span>IPv{ipv6 ? "6" : "4"}: </span>
        <div class="join join-horizontal grow flex">
            <select
                class="select join-item grow"
                disabled={dbs.loaded.length < 2}
                onchange={(ev) => database.setSelected(ev.currentTarget.value)}
            >
                {#if dbs.loaded.length === 0}
                    <option disabled selected>None Loaded</option>
                {/if}

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
                class="btn btn-error join-item">Unload</button
            >
        </div>
    </div>
{/snippet}
