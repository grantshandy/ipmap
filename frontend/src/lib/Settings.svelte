<script lang="ts">
    import {
        stopCapturing,
        startCapturing,
        listDevices,
        loadDatabase,
        builtinDatabaseInfo,
    } from "../utils";

    let dbLoading: boolean = true;
    loadDatabase().then(() => (dbLoading = false));

    let settings_modal: any;

    let capturing: boolean = false;
    let device: string | null = null;
    let database: string | null = null;
</script>

<div class="space-x-2 flow-root">
    <select
        class="select select-bordered select-sm max-w-xs"
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

    <select
        class="select select-bordered select-sm max-w-xs"
        disabled={!device || dbLoading}
        bind:value={database}
    >
        <option disabled selected value={null}>Select Database</option>
        {#await builtinDatabaseInfo() then info}
            {#if info}
                <option selected value="builtin">{info.filename}</option>
            {/if}
        {/await}
    </select>

    {#if !database}
        <button class="btn btn-sm" on:click={settings_modal.showModal()}
            >?</button
        >
    {/if}

    <dialog bind:this={settings_modal} class="modal">
        <div class="modal-box">
            <h3 class="font-bold text-lg">Database Info</h3>
            <p class="py-4">
                {#await builtinDatabaseInfo() then info}
                    {#if info}
                        Filename: {info.filename}
                        Built: {info.built}
                        Attribution: {@html info.attribution}
                    {/if}
                {/await}
            </p>
        </div>
        <form method="dialog" class="modal-backdrop">
            <button>close</button>
        </form>
    </dialog>

    <button class="btn btn-sm btn-secondary" disabled={!device || dbLoading}
        >Load</button
    >

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
