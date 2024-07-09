<script lang="ts">
    import { lookupDns, lookupIpRange } from "../bindings";
    import { type IpLocation } from "../stores/map";
    import { database } from "../stores/database";

    const countryNames = new Intl.DisplayNames("en", { type: "region" });

    export let selection: IpLocation;
</script>

<h2 class="text-lg font-bold">IP Location Info</h2>
<p>
    Location:

    {#if selection.info.city}
        {selection.info.city},
    {/if}
    {#if selection.info.state}
        {selection.info.state},
    {/if}
    {countryNames.of(selection.info.country_code)}
</p>
<hr />
{#each selection.ips as ip}
    <h3 class="font-semibold">{ip}:</h3>
    {#await lookupDns(ip) then dns}
        {#if dns}
            <p>Domain: <span class="code">{dns}</span></p>
        {/if}
    {/await}
    {#await lookupIpRange(ip, $database) then range}
        {#if range}
            <p>
                Block
                <span class="code break-words">
                    {range.lower}
                </span>
                to
                <span class="code">
                    {range.upper}
                </span>
            </p>
        {/if}
    {/await}
{/each}
