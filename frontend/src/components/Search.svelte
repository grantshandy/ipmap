<script lang="ts">
    import { validateIp } from "../bindings";
    import { map } from "../stores/map";
    import Map from "./Map.svelte";

    let query = "";
    let queryValid: boolean = true;

    $: validateIp(query).then((valid) => (queryValid = valid));

    const search = () => map.setSearchIp(query);
</script>

<div class="grow flex flex-col">
    <h1>Search</h1>

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

    <Map />
</div>
