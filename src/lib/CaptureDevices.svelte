<script lang="ts">
    import { commands, type ConnectionInfo, type Device } from "../bindings";
    import { captureError, pcap, connections, refreshConnections, } from "./stores.svelte";

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
        return +((size / Math.pow(1024, i)).toFixed(2)) * 1 + ' ' + ['B', 'kB', 'MB', 'GB', 'TB'][i];
    }
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

    <div class="overflow-y-auto space-y-3">
        <h2>Active Connections</h2>
        {@render renderConnections(connections.active)}
        <hr />
        <h2>
            All Connections
            <button class="btn btn-sm" onclick={refreshConnections}>
                reload
            </button>
        </h2>
        {@render renderConnections(connections.all)}
    </div>
{/if}

{#snippet renderConnections(conn: ConnectionInfo[])}
<div class="overflow-x-auto">
    <table class="table table-xs">
        <thead>
            <tr>
                <th></th>
                <th>IP</th>
                <th>Bytes Down</th>
                <th>Bytes Up</th>
            </tr>
        </thead>
        <tbody>
            {#each conn as a, i}
                <tr>
                    <th>{i + 1}</th>
                    <td>{a.ip}</td>
                    <td>{humanFileSize(a.in_size)}</td>
                    <td>{humanFileSize(a.out_size)}</td>
                </tr>
            {/each}
        </tbody>
    </table>
</div>
{/snippet}