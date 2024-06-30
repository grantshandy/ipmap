<script lang="ts">
    import CloseIcon from "./CloseIcon.svelte";

    import "leaflet/dist/leaflet.css";
    import { map, ApplicationMode } from "../map";
    import { lookupDns } from "../bindings";
    import { fly } from "svelte/transition";

    export let query: string;
    export let state: ApplicationMode;
    const countryNames = new Intl.DisplayNames("en", { type: "region" });

    const mapAction = (container: HTMLDivElement) => {
        map.setContainer(container, state);

        return {
            destroy: () => {
                if ($map.instance) {
                    $map.instance.remove();
                    $map.instance = null;
                }
            },
        };
    };

    const search = (ip: string) => {
        query = ip;
        state = ApplicationMode.SEARCH;
    };
</script>

<svelte:window on:resize={map.resizeMap} />

<div class="relative w-full h-full rounded-md overflow-hidden">
    <div
        class="w-full h-full z-20 select-none overflow-hidden"
        use:mapAction
    ></div>
    {#if $map.selection}
        <div
            in:fly={{ duration: 300, x: 20 }}
            out:fly={{ duration: 300, x: 20 }}
            class="absolute right-0 top-0 bottom-0 z-40 flex flex-col w-64 pl-4 pr-2 py-4 space-y-2 bg-base-100/[0.8] overflow-x-auto"
        >
            <div class="flow-root font-bold">
                <h2 class="float-left">Location Information</h2>
                <button
                    class="float-right btn btn-xs btn-circle"
                    on:click={() => map.setSelection(null)}
                >
                    <CloseIcon />
                </button>
            </div>

            <ul>
                {#if $map.selection.loc.city}
                    <li>City: {$map.selection.loc.city}</li>
                {/if}
                {#if $map.selection.loc.country_code}
                    <li>
                        Country: {countryNames.of(
                            $map.selection.loc.country_code,
                        )}
                    </li>
                {/if}
            </ul>

            {#if state == ApplicationMode.CAPTURE}
                <p>Addresses</p>
                <ul class="list-disc ml-4">
                    {#each $map.selection.ips as ip}
                        <li>
                            {ip}
                            <button class="text-xs" on:click={() => search(ip)}
                                >view</button
                            >
                            {#await lookupDns(ip) then dns}
                                {#if dns}<span class="text-sm">({dns})</span
                                    >{/if}
                            {/await}
                        </li>
                    {/each}
                </ul>
            {/if}
        </div>
    {/if}
</div>
