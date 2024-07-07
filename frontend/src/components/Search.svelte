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
    import { fly } from "svelte/transition";

    const countryNames = new Intl.DisplayNames("en", { type: "region" });

    let query = "";
    let error: string | null = "asdf";

    $: validateIp(query).then(async (valid) => {
        if (query.length == 0) {
            error = null;
            map.setSearchIp(null);
            return;
        }

        if (!valid) {
            error = "Invalid Address";
            map.setSearchIp(null);
            return;
        }

        const loc = await lookupIp(query, $database);

        if (!loc) {
            error = "IP Not Found in Database";
            map.setSearchIp(null);
            return;
        }

        error = null;
        map.setSearchIp(query);
    });

    $: if (!error || error) setTimeout(() => map.invalidateSize(), 350);
</script>

<div class="grow flex flex-col space-y-3">
    <div class="flex space-x-3 items-center">
        <input
            class="input input-sm input-bordered"
            class:border-error={error}
            placeholder="IPv4 Address"
            bind:value={query}
        />

        {#if error}
            <p class="grow text-right text-error text-sm italic">
                Error: {error}
            </p>
        {/if}
    </div>

    <div class="grow flex space-x-2">
        <MapView />
        {#if $map?.selection}
            <div
                transition:fly={{ x: 20, duration: 200 }}
                class="w-1/4 py-2 px-4 space-y-2 select-none"
            >
                <h2 class="text-lg font-bold">IP Location Info</h2>
                <p>
                    Location:

                    {#if $map.selection.info.city}
                        {$map.selection.info.city},
                    {/if}
                    {#if $map.selection.info.state}
                        {$map.selection.info.state},
                    {/if}
                    {countryNames.of($map.selection.info.country_code)}
                </p>
                <hr />
                {#each $map.selection.ips as ip}
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
        {/if}
    </div>
</div>
