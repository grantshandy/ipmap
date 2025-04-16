<script lang="ts">
    import { openUrl } from "@tauri-apps/plugin-opener";
    import {
        commands,
        db,
        type DatabaseStateInfo,
        openDatabaseDialog,
    } from "../bindings";

    const DB_DOWNLOAD_URL =
        "https://github.com/sapics/ip-location-db?tab=readme-ov-file#city";

    const changeDatabase = (ev: Event, dbs: DatabaseStateInfo) => {
        const newDb = dbs.loaded.find(
            (db) => db.name === (ev.target as HTMLSelectElement).value,
        );

        if (newDb) {
            commands.setSelectedDatabase(newDb);
        }
    };
</script>

{#snippet databaseSelector(dbs: DatabaseStateInfo, ipv6: boolean)}
    <div class="flex space-x-4 items-center grow">
        <span>IPv{ipv6 ? "6" : "4"}: </span>
        <div class="join join-horizontal grow flex">
            <select
                class="select join-item grow"
                disabled={dbs.loaded.length < 2}
                onchange={(ev) => changeDatabase(ev, dbs)}
            >
                {#if dbs.loaded?.length == 0}
                    <option disabled selected>None Loaded</option>
                {/if}

                {#each dbs.loaded as db}
                    <option value={db.name} selected={db == dbs.selected}
                        >{db.name}</option
                    >
                {/each}
            </select>
            <button
                disabled={dbs.selected == null}
                onclick={() => {
                    if (dbs.selected) commands.unloadDatabase(dbs.selected);
                }}
                class="btn btn-error join-item">Unload</button
            >
        </div>
    </div>
{/snippet}

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
            onclick={openDatabaseDialog}
            class="btn btn-primary float-right"
            disabled={db.loading != null}
        >
            {#if db.loading}
                <span class="loading loading-spinner loading-xs"></span>
                Loading...
            {:else}
                Load
            {/if}
        </button>
    </div>

    {#if db.ipv4.loaded.length > 0}
        {@render databaseSelector(db.ipv4, false)}
    {/if}

    {#if db.ipv6.loaded.length > 0}
        {@render databaseSelector(db.ipv6, true)}
    {/if}
</div>
