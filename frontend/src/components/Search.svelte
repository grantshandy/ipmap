<script lang="ts">
    import { validateIp } from "../bindings";
    import { map } from "../stores/map";
    import MapView from "./MapView.svelte";

    const countryNames = new Intl.DisplayNames("en", { type: "region" });

    let query = "";
    let queryValid: boolean = true;

    $: validateIp(query).then((valid) => (queryValid = valid));

    const search = () => map.setSearchIp(query);

    $: console.log($map?.selection);
</script>

<div class="grow flex flex-col space-y-3">
    <div>
        <input
            class="input input-sm input-bordered"
            class:border-error={!queryValid}
            placeholder="IPv4 Address"
            bind:value={query}
        />
        <button
            class="btn btn-sm btn-primary"
            disabled={!queryValid}
            on:click={search}
        >
            Search
        </button>
    </div>

    <div class="grow flex relative">
        <MapView />
        {#if $map?.selection}
            <div
                class="absolute right-0 top-0 bottom-0 z-40 w-64 pl-4 pr-2 py-4 space-y-2 bg-base-100/[0.8] overflow-x-auto"
            >
                <h2>Info</h2>
                <p>
                    {($map.selection.info.city
                        ? $map.selection.info.city + ", "
                        : "") +
                        countryNames.of($map.selection.info.country_code)}
                </p>
            </div>
        {/if}
    </div>
</div>
