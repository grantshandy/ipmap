<script lang="ts">
	import "./tailwind.css";
	import "ol/ol.css";

	import { onMount } from "svelte";

	import Map from "ol/Map";
	import View from "ol/View";
	import { OSM } from "ol/source";
	import { Tile as TileLayer, Vector as VectorLayer } from "ol/layer";
	import type VectorSource from "ol/source/Vector";

	let map: Map | null = null;
	export let source: VectorSource;

	onMount(() => {
		map = new Map({
			target: "map",
			layers: [
				new TileLayer({ source: new OSM() }),
				new VectorLayer({ source }),
			],
			view: new View({
				center: [0, 0],
				zoom: 2,
			}),
		});
	});
</script>

<div class="grow" id="map" />
