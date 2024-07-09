<script lang="ts">
    import {
        lookupDns,
        lookupIp,
        lookupIpRange,
        validateIp,
    } from "../bindings";
    import { database } from "../stores/database";
    import { map } from "../stores/map";
    import MapView from "./MapView.svelte";

    const countryNames = new Intl.DisplayNames("en", { type: "region" });

    let query = "";
    let error: string | null = "asdf";

    $: validateAndSearch(query, true);
    $: if (!error || error) setTimeout(() => map.invalidateSize(), 10);

    let searchTimeout: number;
    const validateAndSearch = async (ip: string, pause: boolean) => {
        if (pause) clearTimeout(searchTimeout);

        if (ip.length == 0) {
            error = null;
            map.setSearchIp(null);
            map.resetView();
            return;
        }

        if (!(await validateIp(ip))) {
            error = "Invalid Address";
            map.setSearchIp(null);
            return;
        }

        if (!(await lookupIp(ip, $database))) {
            error = "IP Not Found in Database";
            map.setSearchIp(null);
            return;
        }

        error = null;
        if (pause) {
            searchTimeout = setTimeout(() => map.setSearchIp(ip), 300);
        } else {
            map.setSearchIp(ip);
        }
    };

    let selection: IpLocation | null = null;
    $: selection = $map?.selection;
</script>

<div class="grow flex space-x-2">
    <MapView />
    <div class="w-1/4 space-y-2 bg-base-200 p-2 rounded-box">
        <input
            class="grow input input-sm input-bordered w-full"
            class:border-error={error}
            placeholder="IPv4 Address"
            bind:value={query}
        />
        {#if error}
            <p class="grow text-error text-sm italic font-bold p-2">{error}</p>
        {/if}
        {#if selection}
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
        {/if}
    </div>
</div>
