<script lang="ts">
    import {
        stopCapturing,
        startCapturing,
        listDevices,
        loadInternalDatabase,
    } from "../utils";

    let dbLoading: boolean = true;
    loadInternalDatabase().then(() => (dbLoading = false));

    let capturing: boolean = false;
    let device: string | null = null;
</script>

<div class="space-x-2 flow-root">
    <select
        class="float-left select select-bordered select-sm max-w-xs"
        disabled={dbLoading}
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

    {#if dbLoading}
        <span class="loading loading-spinner loading-md"></span>
    {/if}

    <div class="flex float-right space-x-2">
        <button
            class="btn btn-sm"
            disabled={!device || dbLoading}
            on:click={() => {
                if (device) {
                    stopCapturing(device);
                }

                capturing = false;
                device = null;
            }}
        >
            Reset
        </button>

        <button
            class="btn btn-sm"
            class:btn-success={!capturing}
            class:btn-error={capturing}
            disabled={!device || dbLoading}
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
