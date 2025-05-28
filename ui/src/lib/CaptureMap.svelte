<script lang="ts">
    import MapView from "./Map.svelte";
    import { Marker, type Map } from "leaflet";

    let map: Map | null = $state(null);

    // let markers: Marker[] = [];


</script>

<div class="w-[500px] h-[500px] flex">
    <MapView bind:map={map} />
</div>