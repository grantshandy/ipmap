<script lang="ts">
    import MapView from "./MapView.svelte";
    import {
    currentConnections,
        listDevices,
        startCapturing,
        stopCapturing,
        type ThreadID,
    } from "../bindings";
    import { map } from "../stores/map";
    import { onDestroy } from "svelte";

    const POLL_MS = 500;

    let device: string | null = null;
    let capturing: ThreadID | null = null;

    const toggleCapturing = async () => {
        if (!device) return;

        if (capturing) {
            await stopCapturing(capturing);
            capturing = null;
        } else {
            capturing = await startCapturing(device);
            connLoop();
        }
    };

    const connLoop = async () => {
        if (!capturing) {
            map.setArcState([]);
            return;
        }

        map.setArcState(await currentConnections());

        setTimeout(connLoop, POLL_MS);
    };

    onDestroy(() => {
        if (capturing) stopCapturing(capturing);
    })
</script>

<div class="grow flex flex-col space-y-3">
    <div>
        <select
            bind:value={device}
            disabled={capturing != null}
            class="select select-sm select-bordered w-xs"
        >
            <option disabled selected value={null}>Select Network Device</option
            >
            {#await listDevices() then devices}
                {#each devices as device}
                    <option value={device.name}>
                        {device.desc ?? `${device.name} (No Description)`}
                        {device.prefered ? " (Default)" : ""}
                    </option>
                {/each}
            {/await}
        </select>

        <button
            on:click={toggleCapturing}
            disabled={!device}
            class="btn btn-sm"
            class:btn-primary={!capturing}
            class:btn-error={capturing}
        >
            {#if capturing}
                Stop
            {:else}
                Start
            {/if}

            Capturing
        </button>
    </div>

    <MapView />
</div>
