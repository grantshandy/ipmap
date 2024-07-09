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
    <div class="flex space-x-3 select-none">
        <select
            bind:value={device}
            disabled={capturing != null}
            class="select select-sm select-bordered w-1/3"
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

        <div class="grow flex justify-end space-x-2 text-sm font-semibold">
            <div
                class="flex items-center space-x-2 px-2 rounded-box bg-base-200"
            >
                <div class="rounded-full w-3 h-3 bg-success" />
                <span>Incoming</span>
            </div>
            <div
                class="flex items-center space-x-2 px-2 rounded-box bg-base-200"
            >
                <div class="rounded-full w-3 h-3 bg-error" />
                <span>Outgoing</span>
            </div>
            <div
                class="flex items-center space-x-2 px-2 rounded-box bg-base-200"
            >
                <div class="rounded-full w-3 h-3 bg-warning" />
                <span>Mixed</span>
            </div>
        </div>
    </div>

    <MapView />
</div>
