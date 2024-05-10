<script lang="ts">
    import { onMount } from "svelte";
    import {
        setDatabase,
        setDevice,
        stopCapturing,
        startCapturing,
        listDevices,
    } from "../utils";
    import { open } from "@tauri-apps/api/dialog";
    import { basename } from "@tauri-apps/api/path";
    import { fade } from "svelte/transition";

    const resetBackend = () => {
        setDatabase().then(() => (database = null));
        stopCapturing()
            .then(() => setDevice())
            .then(() => (deviceSet = false));
    };

    // the path to the CSV
    let database: string | null = null;
    let loadingDatabase: boolean = false;

    let capturing: boolean = false;
    let deviceSet: boolean = false;

    // if the page was previously loaded with a device, revert that so it can quietly cancel in the background.
    onMount(resetBackend);
</script>

<div class="space-x-2 flow-root">
    <div class="float-left h-full flex space-x-2">
        <!-- Database File Selector -->
        <button
            class="btn btn-sm btn-primary"
            disabled={loadingDatabase}
            on:click={() => {
                loadingDatabase = true;

                open({
                    title: "Select Database File",
                    directory: false,
                    multiple: false,
                    filters: [
                        {
                            name: "GeoIP Database",
                            extensions: ["csv"],
                        },
                    ],
                }).then((path) => {
                    if (typeof path == "string") {
                        setDatabase(path).then(() => {
                            loadingDatabase = false;
                            database = path;
                        });
                    } else {
                        loadingDatabase = false;
                    }
                });
            }}
        >
            {#if database == null}
                Select File
            {:else}
                {#await basename(database) then filename}
                    {filename}
                {/await}
            {/if}
        </button>

        {#await listDevices() then devices}
            <select
                class="select select-bordered select-sm w-full max-w-xs"
                disabled={!database}
                on:change={(event) => {
                    if (event.target instanceof HTMLSelectElement) {
                        setDevice(event.target.value).then(
                            () => (deviceSet = true),
                        );
                    }
                }}
            >
                <option disabled selected>Select Network Capture Device</option>
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
            </select>
        {/await}

        {#if loadingDatabase}
            <div
                transition:fade={{ duration: 300 }}
                class="loading loading-lg loading-spinner text-secondary"
            ></div>
        {/if}
    </div>

    <div class="flex float-right space-x-2">
        <button
            class="btn btn-sm"
            disabled={!database && !deviceSet}
            on:click={resetBackend}
        >
            Reset
        </button>

        <button
            class="btn btn-sm"
            class:btn-success={!capturing}
            class:btn-error={capturing}
            disabled={!database && !deviceSet}
            on:click={() => {
                if (capturing) {
                    stopCapturing();
                } else {
                    startCapturing().catch(() => (capturing = false));
                }

                capturing = !capturing;
            }}
        >
            {capturing ? "Stop" : "Start"} Capturing
        </button>
    </div>
</div>
