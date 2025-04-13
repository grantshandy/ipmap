<script lang="ts">
    import {
        commands,
        type ConnectionInfo,
        type Device,
        captureError,
        pcap,
        refreshConnections,
    } from "../bindings";

    let device: Device | null = $state(null);

    $effect(() => {
        if (
            pcap.state == null ||
            typeof pcap.state != "object" ||
            pcap.state.devices.length == 0
        )
            return;

        if (device == null) {
            device = pcap.state.devices[0];
        } else {
            device =
                pcap.state.devices.find((d) => d.name == device?.name) ?? null;
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

    const startCapture = async () => {
        if (!device) return;
        captureError(commands.startCapture(device));
    };

    const stopCapture = () => captureError(commands.stopCapture());

    const humanFileSize = (size: number): string => {
        const i = size == 0 ? 0 : Math.floor(Math.log(size) / Math.log(1024));
        return (
            +(size / Math.pow(1024, i)).toFixed(2) * 1 +
            " " +
            ["B", "kB", "MB", "GB", "TB"][i]
        );
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

        {#if pcap.state.capture == null}
            <button
                class="join-item btn btn-primary"
                onclick={startCapture}
                disabled={device == null}>Start Capture</button
            >
        {:else}
            <button class="join-item btn btn-error" onclick={stopCapture}
                >Stop Capture</button
            >
        {/if}
    </div>
{/if}

{#if pcap.connections}
        <h2>Active Connections</h2>
    <ol class="list-decimal">
        {#each pcap.connections.active as a}
            <li>
                <span>{a.ip}:</span>
                <ul class="list-disc">
                    <li>In: {humanFileSize(a.in.bytes_per_second)}/s ({humanFileSize(a.in.size)})</li>
                    <li>Out: {humanFileSize(a.out.bytes_per_second)}/s ({humanFileSize(a.out.size)})</li>
                </ul>
            </li>
        {/each}
    </ol>
{/if}
