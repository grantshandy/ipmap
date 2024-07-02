<script lang="ts">
    import { onDestroy } from "svelte";
    import {
        listDevices,
        startCapturing,
        stopCapturing,
    } from "../bindings";
    import { map } from "../map";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import type { Connection, DatabaseInfo } from "../bindings";

    export let database: DatabaseInfo | null;
    export let loading: string | null;

    let device: string | null = null;
    let capturing: string | null = null;

    const toggleCapturing = async () => {
        if (!device) return;

        if (capturing) {
            stopCapturing(capturing);
            capturing = null;
        } else {
            capturing = await startCapturing(device).catch(() => null);
        }
    };

    let captureStop: UnlistenFn;
    listen("new_capture", (event) => {
        const connection: Connection = event.payload as Connection;

        if (database && connection.thread_id == capturing) {
            map.addCaptureIp(connection, database);
        }
    }).then((x) => (captureStop = x));

    const cleanup = () => {
        if (capturing) stopCapturing(capturing);
        if (captureStop) captureStop();
    };
    onDestroy(cleanup);
    window.onbeforeunload = cleanup;
</script>

<select
    class="select select-bordered select-sm max-w-xs"
    disabled={!database || loading != null}
    bind:value={device}
>
    <option selected disabled value={null}>Select Capture Device</option>
    {#await listDevices() then devices}
        {#each devices as option}
            <option value={option.name}>
                {option.desc ?? "No Description"}
                {option.prefered ? "(Default)" : ""}
            </option>
        {/each}
    {/await}
</select>

<button
    class="btn btn-sm"
    class:btn-primary={!capturing}
    class:btn-error={capturing}
    disabled={!device || !database || loading != null}
    on:click={toggleCapturing}
>
    {capturing ? "Stop" : "Start"} Capturing
</button>
