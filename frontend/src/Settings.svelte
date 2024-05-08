<script>
    import { onMount } from "svelte";
    import {
        setDatabase,
        setDevice,
        stopCapturing,
        startCapturing,
        listDevices,
    } from "./bridge";
    import { open } from "@tauri-apps/api/dialog";
    import { basename } from "@tauri-apps/api/path";
    import { fade } from "svelte/transition";

    const resetBackend = () => {
        setDatabase().then(() => (database = null));
        stopCapturing().then(() => setDevice());
    };

    // the path to the CSV
    let database = null;
    let loadingDatabase = false;

    let capturing = false;
    let deviceSet = false;

    // if the page was previously loaded with a device, revert that so it can quietly cancel in the background.
    onMount(resetBackend);
</script>

<div class="space-x-2 flow-root">
    <!-- Database File Selector -->
    <button
        class="float-left btn btn-sm btn-primary"
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
        {#if loadingDatabase}
            Loading...
        {:else if database != null}
            {#await basename(database) then filename}
                {filename}
            {/await}
        {:else}
            Select File
        {/if}
    </button>

    {#await listDevices() then devices}
        <select
            class="float-left select select-bordered select-sm w-full max-w-xs"
            disabled={!database}
            on:change={(event) =>
                setDevice(event.target.value).then(() => (deviceSet = true))}
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
        <span
            transition:fade={{ duration: 300 }}
            class="loading loading-spinner text-neutral"
        ></span>
    {/if}

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
