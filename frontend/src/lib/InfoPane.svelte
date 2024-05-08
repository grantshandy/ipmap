<script>
    import { fly } from "svelte/transition";

    export let selection;

    $: console.log(selection);
</script>

{#if selection}
    <div
        in:fly={{ x: -20, duration: 400 }}
        out:fly={{ x: 20, duration: 400 }}
        class="flex flex-col rounded-sm pl-4 py-4 space-y-2 w-64"
    >
        <div class="w-full flow-root">
            <p class="float-left">Location Information</p>
            <button
                on:click={() => (selection = null)}
                class="float-right btn btn-xs">X</button
            >
        </div>

        <ul>
            {#if selection.loc.city}
                <li>City: {selection.loc.city}</li>
            {/if}
            {#if selection.loc.state}
                <li>State: {selection.loc.state}</li>
            {/if}
            {#if selection.loc.country_code}
                <li>Country: {selection.loc.country_code}</li>
            {/if}
        </ul>

        <p>Addresses</p>
        <ul class="list-disc ml-4">
            {#each selection.ips as ip}
                <li>{ip}</li>
            {/each}
        </ul>
    </div>
{/if}
