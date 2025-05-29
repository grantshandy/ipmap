<script lang="ts">
    import {
        type ConnectionInfo,
        type MovingAverageInfo,
        pcap,
        database,
    } from "../bindings";

    let sortedConnections: [string, ConnectionInfo][] = $derived(
        Object.entries(pcap.connections).sort(
            ([, a], [, b]) =>
                b.down.avg_s + b.up.avg_s - (a.down.avg_s + a.up.avg_s),
        ),
    );

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
</script>

{#if pcap.connections}
    <h2>Active Connections</h2>
    <ol class="list-decimal ml-4">
        {#each sortedConnections as [ip, info] (ip)}
            <li class="pl-8">
                <span>{ip}:</span>
                <ul class="list-disc">
                    <li>Down: {connState(info.down)}</li>
                    <li>Up: {connState(info.up)}</li>
                    {#if database.ipv4.loaded || database.ipv6.loaded}
                        {#await database.lookupIp(ip) then res}
                            {#if res}
                                <li>{res.loc.city}, {res.loc.country_code}</li>
                            {/if}
                        {/await}
                    {/if}
                </ul>
            </li>
        {/each}
    </ol>
{/if}
