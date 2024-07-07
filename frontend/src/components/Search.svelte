<script lang="ts">
    import { fly } from "svelte/transition";
    import { lookupIp, validateIp } from "../bindings";
    import { database } from "../stores/database";
    import { map } from "../stores/map";
    import IpLocationView from "./IpLocationView.svelte";
    import MapView from "./MapView.svelte";

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
    <div class="w-1/4 space-y-2">
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
            <div transition:fly={{ x: 20, duration: 200 }}>
                <IpLocationView loc={$map?.selection} />
            </div>
        {/if}
    </div>
</div>
