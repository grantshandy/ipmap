<script lang="ts">
    import { pcap, database } from "$lib/../bindings";
    import MapView from "$lib/Map.svelte";
    import { type Map, marker, type Marker } from "leaflet";

    let map: Map | null = $state(null);

    const markers: Record<string, Marker> = {};

    pcap.onConnStart(async (ip, _) => {
        if (!map) return;

        const loc = await database.lookupIp(ip);

        if (!loc) {
            // TODO: handle case where location is not found
            return;
        }

        markers[ip] = marker(loc.crd)
            .bindPopup(ip)
            .addTo(map);
    });

    pcap.onConnEnd((ip) => {
        markers[ip]?.remove();
        delete markers[ip];
    });
</script>

<div class="grow flex flex-col space-y-3">
    {#if pcap.status != null && typeof pcap.status !== "string"}
        <div class="w-full flow-root">
            <div class="float-left join join-horizontal">
                <select
                    class="select select-sm join-item"
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
                    <button
                        class="join-item btn btn-sm"
                        onclick={pcap.stopCapture}
                    >
                        Stop Capture
                    </button>
                {:else}
                    <button
                        class="join-item btn btn-sm btn-primary"
                        onclick={pcap.startCapture}
                        disabled={pcap.device == null}
                    >
                        Start Capture
                    </button>
                {/if}
            </div>
            {#if pcap.status.capture != null}
                <p class="float-right">
                    No. Connections: {Object.keys(pcap.connections).length}
                </p>
            {/if}
        </div>

        <div class="grow flex">
            <MapView bind:map />
        </div>
    {/if}
</div>
