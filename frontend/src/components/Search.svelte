<script lang="ts">
    import { fly } from "svelte/transition";
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
    $: if (!error || error) setTimeout(() => map.invalidateSize(), 350);

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
        {#if $map?.selection}
            <div transition:fly={{ x: 20, duration: 200 }} class="space-y-2">
                <h2 class="text-lg font-bold">IP Location Info</h2>
                <p>
                    Location:

                    {#if $map?.selection.info.city}
                        {$map?.selection.info.city},
                    {/if}
                    {#if $map?.selection.info.state}
                        {$map?.selection.info.state},
                    {/if}
                    {countryNames.of($map?.selection.info.country_code)}
                </p>
                <hr />
                {#each $map?.selection.ips as ip}
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
            </div>
        {/if}
    </div>
</div>
