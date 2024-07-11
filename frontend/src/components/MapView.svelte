<script lang="ts">
    import "leaflet/dist/leaflet.css";
    import { map } from "../stores/map";
    import { darkTheme, theme } from "../stores";

    const mapAction = (cont: HTMLDivElement) => {
        map.init(cont);

        return {
            destroy: () => map.deinit(),
        };
    };
</script>

<div class="grow relative rounded-box select-none">
    {#if $map}
        <div
            class="absolute top-2 left-2 z-30 join join-vertical"
        >
            <button
                on:click={() => $map.inst.zoomIn()}
                class="btn btn-sm font-bold text-xl join-item">+</button
            >
            <button
                on:click={() => $map.inst.zoomOut()}
                class="btn btn-sm font-bold text-xl join-item">&#x2212;</button
            >
        </div>
    {/if}
    <slot />
    <div
        use:mapAction
        class="w-full h-full z-20"
        class:map-dark={$theme == darkTheme}
    ></div>
</div>
