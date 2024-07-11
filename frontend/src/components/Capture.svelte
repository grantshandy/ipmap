<script lang="ts">
    import MapView from "./MapView.svelte";
    import { type Device, type ThreadID, capture } from "../bindings";
    import { map } from "../stores/map";
    import { onDestroy } from "svelte";
    import type { UnlistenFn } from "@tauri-apps/api/event";

    const POLL_MS = 250;

    let device: Device | null = null;
    let capturing: { id: ThreadID; unlisten: UnlistenFn } | null = null;

    const toggleCapturing = async () => {
        if (!device) return;

        if (capturing) {
            await capture.stopCapturing(capturing.id);
            capturing = null;
        } else {
            const unlisten = await capture.onNewConnection((ip) => {
                if (!capturing) unlisten();
                map.addIp(ip);
            });

            capturing = {
                id: await capture.startCapturing(device),
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

        capture.currentConnections().then(map.setArcState);
        setTimeout(currentConnectionLoop, POLL_MS);
    };

    const cleanup = () => {
        if (capturing) {
            console.log("stopping capture of " + capturing.id);
            capture.stopCapturing(capturing.id);
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
            {#await capture.listDevices() then devices}
                {#each devices as device}
                    <option value={device}>
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

    <MapView>
        <div
            class="absolute left-0 bottom-0 z-30 flex items-center pr-1 pt-0.5 text-xs rounded-tr-md bg-base-100"
        >
            <div class="color-indicator bg-success" />
            <span>Incoming</span>
            <div class="color-indicator bg-error" />
            <span>Outgoing</span>
            <div class="color-indicator bg-warning" />
            <span>Mixed</span>
        </div>
    </MapView>
</div>

<style lang="postcss">
    .color-indicator {
        @apply rounded-full w-3 h-3 ml-1.5 mr-0.5;
    }
</style>
