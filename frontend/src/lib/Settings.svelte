<script lang="ts">
    import { stopCapturing, startCapturing, listDevices } from "../utils";

    let capturing: boolean = false;
    let device: string | null = null;

    $: console.log(device);

    // if the page was previously loaded with a device, revert that so it can quietly cancel in the background.
</script>

<div class="space-x-2 flow-root">
    <div class="float-left h-full flex space-x-2">
        {#await listDevices() then devices}
            <select
                class="select select-bordered select-sm w-full max-w-xs"
                bind:value={device}
            >
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
    </div>

    <div class="flex float-right space-x-2">
        <button
            class="btn btn-sm"
            disabled={!device}
            on:click={() => (device = null)}
        >
            Reset
        </button>

        <button
            class="btn btn-sm"
            class:btn-success={!capturing}
            class:btn-error={capturing}
            disabled={!device}
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
