<script lang="ts">
    import { basename } from "@tauri-apps/api/path";
    import {
        stopCapturing,
        startCapturing,
        listDevices,
        loadDatabase,
        type DatabaseInfo,
        listDatabases,
    } from "../utils";
    import { open } from "@tauri-apps/api/dialog";
    import { onMount } from "svelte";

    let dbLoading: string | null = "Built in DB";
    let loadingModal: HTMLDialogElement;
    $: if ((loadingModal != null && dbLoading) || !dbLoading) {
        if (dbLoading != null) {
            loadingModal.showModal();
        } else {
            loadingModal.close();
        }
    }

    export let database: DatabaseInfo | null;
    let databases: DatabaseInfo[] = [];
    const updateDatabases = async () => (databases = await listDatabases());
    onMount(async () => {
        loadDatabase(null).then(() => (dbLoading = null));
        updateDatabases();
    });

    let device: string | null = null;
    let capturing: boolean = false;

    let settingsModal: HTMLDialogElement;
</script>

<div class="space-x-2 flow-root">
    <select
        class="select select-bordered select-sm max-w-xs"
        disabled={dbLoading != null || capturing}
        bind:value={device}
    >
        <option disabled selected value={null}>Select Network Device</option>
        {#await listDevices() then devices}
            {#each devices as device}
                <option value={device.name}>
                    {#if device.desc != null}
                        {device.desc}
                    {:else}
                        No Description ({device.name})
                    {/if}
                    {#if device.prefered}(Default){/if}
                </option>
            {/each}
        {/await}
    </select>

    <select
        class="select select-bordered select-sm max-w-xs"
        disabled={!device ||
            dbLoading != null ||
            capturing ||
            databases.length == 0}
        bind:value={database}
    >
        <option disabled selected value={null}
            >Select IP Geolocation Database</option
        >
        {#each databases as option}
            <option value={option}>
                {option.name}
            </option>
        {/each}
    </select>

    {#if device && database}
        <button
            class="btn btn-sm btn-secondary"
            disabled={dbLoading != null}
            on:click={() => settingsModal.showModal()}>?</button
        >
        <dialog bind:this={settingsModal} class="modal">
            <div class="modal-box">
                <h3 class="font-bold text-lg">Database Info</h3>
                {#if database}
                    <ul class="py-4 list-dic">
                        <li>{database.name}</li>
                        <li>
                            Distinct Locations: {database.locations.toLocaleString()}
                        </li>
                        {#if database.path}
                            <li>Path: {database.path}</li>
                        {/if}
                        {#if database.attribution_text}
                            <li>Attribution: {database.attribution_text}</li>
                        {/if}
                        <li>Built: {database.build_time}</li>
                    </ul>
                {/if}
            </div>
            <form method="dialog" class="modal-backdrop">
                <button>close</button>
            </form>
        </dialog>
    {/if}

    <button
        class="btn btn-sm btn-secondary"
        disabled={!device || dbLoading != null || capturing}
        on:click={async () => {
            const path = await open({
                directory: false,
                multiple: false,
                filters: [{ name: "IPv4-num database", extensions: ["csv"] }],
            });

            if (path == null) {
                return;
            }

            dbLoading = await basename(path.toString());

            const newDatabase = await loadDatabase(path).catch(() => {
                dbLoading = null;
            });

            if (newDatabase != null) {
                databases = [...databases, newDatabase];
                database = newDatabase;
            }

            updateDatabases();

            dbLoading = null;
        }}>+</button
    >
    <dialog class="modal" bind:this={loadingModal}>
        <div class="modal-box flow-root">
            <p class="float-left">Loading {dbLoading}</p>
            <span class="float-right loading loading-spinner loading-md"></span>
        </div>
        <form method="dialog" class="modal-backdrop"></form>
    </dialog>

    <div class="float-right flex space-x-2">
        <button
            class="btn btn-sm"
            disabled={!device || dbLoading != null || capturing}
            on:click={() => {
                if (device) {
                    stopCapturing(device);
                }

                capturing = false;
                database = null;
                device = null;
            }}
        >
            Reset
        </button>

        <button
            class="btn btn-sm"
            class:btn-success={!capturing}
            class:btn-error={capturing}
            disabled={!device || !database || dbLoading != null}
            on:click={() => {
                if (device) {
                    if (capturing) {
                        stopCapturing(device);
                    } else {
                        startCapturing(device).catch(() => (capturing = false));
                    }
                }

                capturing = !capturing;
            }}
        >
            {capturing ? "Stop" : "Start"} Capturing
        </button>
    </div>
</div>
