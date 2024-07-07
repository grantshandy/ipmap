<script lang="ts">
    import { type IpLocation } from "../stores/map";
    import { lookupDns, lookupIpRange } from "../bindings";
    import { database } from "../stores/database";

    export let loc: IpLocation;

    const countryNames = new Intl.DisplayNames("en", { type: "region" });
</script>

<div
    class="py-2 px-4 space-y-2 select-none"
>
    <h2 class="text-lg font-bold">IP Location Info</h2>
    <p>
        Location:

        {#if loc.info.city}
            {loc.info.city},
        {/if}
        {#if loc.info.state}
            {loc.info.state},
        {/if}
        {countryNames.of(loc.info.country_code)}
    </p>
    <hr />
    {#each loc.ips as ip}
        <h3 class="font-semibold">{ip}:</h3>
        {#await lookupDns(ip) then dns}
            {#if dns}
                <p>Domain: <span class="code">{dns}</span></p>
            {/if}
        {/await}
        {#await lookupIpRange(ip, $database) then range}
            {#if range}
                <p>
                    Range:
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
</div>
