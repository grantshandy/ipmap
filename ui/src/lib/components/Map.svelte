<script lang="ts">
  import "leaflet/dist/leaflet.css";
  import "leaflet";
  import {
    LatLngBounds,
    type Map,
    type DivIcon,
    type LatLngExpression,
  } from "leaflet";
  import * as leaflet from "leaflet";
  import "leaflet-edgebuffer";
  import type { Snippet } from "svelte";

  import markerIconUrl from "leaflet/dist/images/marker-icon.png";
  import markerIconRetinaUrl from "leaflet/dist/images/marker-icon-2x.png";
  import markerShadowUrl from "leaflet/dist/images/marker-shadow.png";
  leaflet.Icon.Default.prototype.options.iconUrl = markerIconUrl;
  leaflet.Icon.Default.prototype.options.iconRetinaUrl = markerIconRetinaUrl;
  leaflet.Icon.Default.prototype.options.shadowUrl = markerShadowUrl;
  leaflet.Icon.Default.imagePath = "";

  let { map = $bindable(), children }: { map: Map | null; children?: Snippet } =
    $props();

  export const DEFAULT_POS: LatLngExpression = [25, 0];
  export const DEFAULT_ZOOM = 2;

  export const mkIcon = (count: number | null, active?: boolean): DivIcon =>
    leaflet.divIcon({
      html: `<div class="${active ? "marker-icon-active" : "marker-icon"}">${count ? count : ""}</div>`,
      className: "dummyclass",
      iconSize: active ? [30, 30] : [20, 20],
      iconAnchor: active ? [15, 15] : [10, 10],
    });

  const mapAction = (cont: HTMLDivElement) => {
    map = leaflet.map(cont, {
      preferCanvas: false,
      minZoom: 2,
      maxZoom: 15,
      zoomControl: false,
    });
    map.setView(DEFAULT_POS, DEFAULT_ZOOM);
    map.attributionControl.remove();
    map.setMaxBounds(new LatLngBounds([-150, -300], [150, 400]));
    leaflet
      .tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
        noWrap: true,
        edgeBufferTiles: 2,
        bounds: [
          [-90, -180],
          [90, 180],
        ],
      })
      .addTo(map);

    return {
      destroy: () => map?.remove(),
    };
  };
</script>

<svelte:window on:resize={() => map?.invalidateSize()} />

<div class="overflow-none relative grow">
  <div class="join join-vertical absolute top-2 left-2 z-30 border select-none">
    <button
      onclick={() => map?.zoomIn()}
      class="btn join-item btn-sm text-xl font-bold">+</button
    >
    <button
      onclick={() => map?.zoomOut()}
      class="btn join-item btn-sm text-xl font-bold">&#x2212;</button
    >
  </div>
  {@render children?.()}
  <div use:mapAction class="z-20 h-full w-full select-none"></div>
</div>
