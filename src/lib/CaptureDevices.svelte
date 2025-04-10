<script lang="ts">
    import { commands, events, type Device } from "../bindings";
    import { captureError, pcap } from "./stores.svelte";

    let device: Device | null = $state(null);

    $effect(() => {
        if (pcap.state == null || typeof pcap.state != "object") return;

        if (device == null && pcap.state.devices.length > 0) {
            device = pcap.state.devices[0];
        }

        if (pcap.state.capture != null) {
            for (const d of pcap.state.devices) {
                if (d.name == pcap.state.capture.name) {
                    device = d;
                    break;
                }
            }
        }
    });

    events.newPacket.listen((ev) => console.log(ev.payload));

    const startCapture = async () => {
        if (!device) return;
        captureError(commands.startCapture(device));
    };
</script>

{#if typeof pcap.state == "string"}
    <p>Couldn't load <code>libpcap</code>: <code>{pcap.state}</code></p>
{:else if pcap.state != null}
    <pre>{pcap.state.version}</pre>

    <div class="join join-horizontal">
        <select class="select join-item" bind:value={device}>
            {#each pcap.state.devices as device}
                <option value={device} selected>
                    {device.name}
                    {#if device.description}
                        : ({device.description})
                    {/if}
                </option>
            {/each}
        </select>

        <button
            class="join-item btn btn-primary"
            onclick={startCapture}
            disabled={device == null}>Start Capture</button
        >

        {#if pcap.state.capture != null}
            <button
                class="join-item btn btn-error"
                onclick={() => captureError(commands.stopCapture())}
                >Stop Capture</button
            >
        {/if}
    </div>
{/if}
