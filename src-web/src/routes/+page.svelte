<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";

  import "ol/ol.css";
  import "../tailwind.css";

  import Feature from "ol/Feature.js";
  import Map from "ol/Map.js";
  import Point from "ol/geom/Point.js";
  import View from "ol/View.js";
  import { Circle as CircleStyle, Stroke, Style } from "ol/style.js";
  import { OSM, Vector as VectorSource } from "ol/source.js";
  import { Tile as TileLayer, Vector as VectorLayer } from "ol/layer.js";
  import { easeOut } from "ol/easing.js";
  import { fromLonLat } from "ol/proj.js";
  import { getVectorContext } from "ol/render.js";
  import { unByKey } from "ol/Observable.js";

  const source = new VectorSource();
  const tile = new TileLayer({ source: new OSM() });
  const vector = new VectorLayer({ source });
  let map: Map | null = null;

  let locations: {
    ip: string;
    lat: number;
    lon: number;
    city: string;
    country: string;
  }[] = [];
  let connectionCount: number = 0;
  let uniqueIps: string[] = [];

  onMount(async () => {
    await listen("connection", async (event: any) => {
      let payload: { ip: string } = event.payload;

      connectionCount++;
      if (!uniqueIps.includes(payload.ip)) {
        console.log("got a connection");
        uniqueIps.push(payload.ip);
        addRandomFeature();
      }
    });

    map = new Map({
      target: "map",
      layers: [
        tile,
        vector,
      ],
      view: new View({
        center: [0, 0],
        zoom: 2,
      }),
    });
  });

  function addRandomFeature() {
    const x = Math.random() * 360 - 180;
    const y = Math.random() * 170 - 85;
    const geom = new Point(fromLonLat([x, y]));

    // source.addFeature(new Feature(geom));
  }


  
  source.on('addfeature', function (e) {
    flash(e.feature);
  });

  const flashDuration = 3000;
  function flash(feature: Feature) {
    const start = Date.now();
    const flashGeom = feature.getGeometry().clone();
    const listenerKey = tile.on('postrender', animate);

    function animate(event) {
      const frameState = event.frameState;
      const elapsed = frameState.time - start;
      if (elapsed >= flashDuration) {
        unByKey(listenerKey);
        return;
      }
      const vectorContext = getVectorContext(event);
      const elapsedRatio = elapsed / flashDuration;
      // radius will be 5 at start and 30 at end.
      const radius = easeOut(elapsedRatio) * 25 + 5;
      const opacity = easeOut(1 - elapsedRatio);

      const style = new Style({
        image: new CircleStyle({
          radius: radius,
          stroke: new Stroke({
            color: 'rgba(255, 0, 0, ' + opacity + ')',
            width: 0.25 + opacity,
          }),
        }),
      });

      vectorContext.setStyle(style);
      vectorContext.drawGeometry(flashGeom);
      // tell OpenLayers to continue postrender animation
      if (map) {
        map.render();
      }
    }
  }
</script>

<div class="w-screen h-screen flex">
  <div class="w-30" />
  <div class="grow flex flex-col">
    {#if window.backendError}
      <div class="flow-root py-1 px-2 bg-red-600 text-gray-50 align-middle">
        <div class="float-left h-full flex items-center">
          <p><span class="font-bold">Error:</span> {window.backendError}</p>
        </div>
        <button class="float-right px-2 py-1 font-semibold border rounded-md"
          >Close</button
        >
      </div>
    {/if}
    <div class="grow" id="map" />
  </div>
</div>
