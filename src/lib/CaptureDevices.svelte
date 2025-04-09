<script lang="ts">
    import { commands, events, type Device } from "../bindings";
    import { captureError, pcapState } from "./stores.svelte";

    let device: Device | null = $state(null);

    $effect(() => {
        if (device == null && pcapState.devices.length > 0) {
            device = pcapState.devices[0];
        }
    });

    events.newPacket.listen((ev) => console.log(ev.payload));

    const startCapture = async () => {
        if (!device) return;
        captureError(commands.startCapture(device));
    };
</script>

{#if pcapState.error}
    <p>Error loading libpcap:</p>
    <pre>{pcapState.error}</pre>
{:else}
    <div class="join join-horizontal">
        <select class="select join-item" bind:value={device}>
            {#each pcapState.devices as device}
                <option value={device} selected>
                    {device.name}
                    {#if device.description}
                        : ({device.description})
                    {/if}
                </option>
            {/each}
        </select>

        <button class="join-item btn btn-primary" onclick={startCapture}
            >Start Capture</button
        >

        {#if pcapState.capturing != null}
            <button
                class="join-item btn btn-error"
                onclick={() => captureError(commands.stopCapture())}
                >Stop Capture</button
            >
        {/if}
    </div>
{/if}
