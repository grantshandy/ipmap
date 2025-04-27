<script lang="ts">
    import {
        type Device,
        type MovingAverageInfo,
        commands,
        pcap,
        db,
        startCapture,
        stopCapture,
    } from "../bindings";

    let device: Device | null = $state(null);

    $inspect(pcap.state);

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

    const humanFileSize = (size: number): string => {
        const i = size == 0 ? 0 : Math.floor(Math.log(size) / Math.log(1024));
        return (
            +(size / Math.pow(1024, i)).toFixed(2) * 1 +
            " " +
            ["B", "kB", "MB", "GB", "TB"][i]
        );
    };

    const connState = (info: MovingAverageInfo): string =>
        `${humanFileSize(info.total)} | ${humanFileSize(info.avg_s)}/s`;

    let dbLoaded = $derived(db.ipv4.selected || db.ipv6.selected);
</script>

{#if typeof pcap.state == "string"}
    <p>Couldn't load <code>libpcap</code>: <code>{pcap.state}</code></p>
{:else if pcap.state != null}
    <pre>{pcap.state.version}</pre>

    <div class="join join-horizontal">
        <select
            class="select join-item"
            disabled={pcap.state.capture != null}
            bind:value={device}
        >
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
                onclick={() => startCapture(device)}
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
    <ol class="list-decimal ml-4">
        {#each Object.entries(pcap.connections).sort(([, a], [, b]) => b.down.avg_s + b.up.avg_s - (a.down.avg_s + a.up.avg_s)) as [ip, info]}
            <li class="pl-8">
                <span>{ip}:</span>
                <ul class="list-disc">
                    <li>Down: {connState(info.down)}</li>
                    <li>Up: {connState(info.up)}</li>
                    {#if dbLoaded}
                        {#await commands.lookupIp(ip) then loc}
                            <li>
                                location: <pre>{JSON.stringify(loc)}</pre>
                            </li>
                        {/await}
                    {/if}
                </ul>
            </li>
        {/each}
    </ol>
{/if}
