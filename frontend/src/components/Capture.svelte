<script lang="ts">
    import MapView from "./MapView.svelte";
    import {
        currentConnections,
        listDevices,
        onNewConnection,
        startCapturing,
        stopCapturing,
        type ThreadID,
    } from "../bindings";
    import { map } from "../stores/map";
    import { onDestroy } from "svelte";
    import type { UnlistenFn } from "@tauri-apps/api/event";

    const POLL_MS = 250;

    let device: string | null = null;
    let capturing: { id: ThreadID; unlisten: UnlistenFn } | null = null;

    const toggleCapturing = async () => {
        if (!device) return;

        if (capturing) {
            await stopCapturing(capturing.id);
            capturing = null;
        } else {
            const unlisten = await onNewConnection((ip) => {
                if (!capturing) unlisten();
                map.addIp(ip);
            });

            capturing = {
                id: await startCapturing(device),
                unlisten,
            };

            currentConnectionLoop();
        }
    };

    const currentConnectionLoop = () => {
        if (!capturing) {
            map.setArcState([]);
            return;
        }

        currentConnections().then(map.setArcState);
        setTimeout(currentConnectionLoop, POLL_MS);
    };

    const cleanup = () => {
        if (capturing) {
            console.log("stopping capture of " + capturing.id);
            stopCapturing(capturing.id);
            capturing.unlisten();
            capturing = null;
        }
    };

    onDestroy(cleanup);
    window.onbeforeunload = cleanup;
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
            {capturing ? "Stop" : "Start"} Capturing
        </button>
    </div>

    <MapView />
</div>
