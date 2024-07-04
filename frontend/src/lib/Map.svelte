<script lang="ts">
    import "leaflet/dist/leaflet.css";
    import { map, ApplicationMode } from "../map";

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
    <div
        class="absolute right-0 top-0 bottom-0 z-40 flex flex-col w-64 pl-4 pr-2 py-4 space-y-2 bg-base-100/[0.8] overflow-x-auto"
    >
        <h2>Current Connections</h2>

        <ul class="ml-2 list-disc">
            {#each $map.current as current}
                <li>{current.ip}</li>
            {/each}
        </ul>
    </div>
</div>
