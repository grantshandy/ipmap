<script lang="ts">
    import "leaflet/dist/leaflet.css";
    import { map } from "../stores/map";

    const mapAction = (cont: HTMLDivElement) => {
        map.init(cont);

        return {
            destroy: () => map.deinit(),
        };
    };
</script>

<div class="grow flex">
    <div use:mapAction class="grow z-20 select-none overflow-hidden"></div>

</div>
