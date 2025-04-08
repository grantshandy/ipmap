<script lang="ts">
    import {
        commands,
        events,
        type AppStateInfo,
        type DatabaseInfo,
        type DatabaseStateInfo,
    } from "../bindings";
    import { open } from "@tauri-apps/plugin-dialog";

    let { appState = $bindable() }: { appState: AppStateInfo } = $props();

    $inspect(appState);

    const updateAppState = (state: AppStateInfo) => {
        console.log("updating...");

        appState.loading = state.loading;

        appState.ipv4.loaded = state.ipv4.loaded;
        appState.ipv4.selected = state.ipv4.selected
            ? appState.ipv4.loaded.filter(
                  (info) => info.path == state.ipv4.selected?.path,
              )[0]
            : null;

        appState.ipv6.loaded = state.ipv6.loaded;
        appState.ipv6.selected = state.ipv6.selected
            ? appState.ipv6.loaded.filter(
                  (info) => info.path == state.ipv6.selected?.path,
              )[0]
            : null;
    };

    commands.databaseState().then(updateAppState);
    events.databaseStateChange.listen((ev) => updateAppState(ev.payload));

    const openDatabase = async () => {
        const file = await open({
            title: "Open IP Geolocation City Database",
            multiple: false,
            directory: false,
            filters: [
                {
                    name: "IP Geolocation City Database",
                    extensions: ["csv", "csv.gz"],
                },
            ],
        });

        if (file) commands.loadDatabase(file);
    };

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
    <div class="flex space-x-4 items-center">
        <span>IPv{ipv6 ? "6" : "4"}: </span>
        <div class="join join-horizontal">
            <select
                class="select join-item"
                disabled={dbs.loaded.length < 2}
                onchange={(ev) => changeDatabase(ev, dbs)}
            >
                {#if dbs.loaded?.length == 0}
                    <option disabled selected>No Database Loaded</option>
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

<div class="flex space-x-4 items-center">
    <button
        onclick={openDatabase}
        class="btn btn-primary join-item"
        disabled={appState.loading != null}>Load</button
    >
    {#if appState.loading}
        <span class="loading loading-spinner"></span>
        <span class="italic text-sm">Loading {appState.loading}...</span>
    {/if}
</div>

{@render databaseSelector(appState.ipv4, false)}
{@render databaseSelector(appState.ipv6, true)}
