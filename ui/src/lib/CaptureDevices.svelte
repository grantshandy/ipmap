<script lang="ts">
    import { pcap } from "../bindings";
</script>

{#if typeof pcap.status == "string"}
    <p>Couldn't load <code>libpcap</code>: <code>{pcap.status}</code></p>
{:else if pcap.status != null}
    <p>Loaded <code>{pcap.status.version}</code></p>

    <div class="join join-horizontal">
        <select
            class="select join-item"
            disabled={pcap.status.capture != null}
            bind:value={pcap.device}
        >
            {#each pcap.status.devices as device}
                <option value={device} selected>
                    {device.name}
                    {#if device.description}
                        : ({device.description})
                    {/if}
                </option>
            {/each}
        </select>

        {#if pcap.status.capture}
            <button class="join-item btn btn-error" onclick={pcap.stopCapture}>
                Stop Capture
            </button>
        {:else}
            <button
                class="join-item btn btn-primary"
                onclick={pcap.startCapture}
                disabled={pcap.device == null}
            >
                Start Capture
            </button>
        {/if}
    </div>
{/if}
