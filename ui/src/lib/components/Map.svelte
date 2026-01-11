<script lang="ts">
  import "leaflet/dist/leaflet.css";
  import "leaflet";
  import { LatLngBounds, type Marker, type Map } from "leaflet";
  import * as leaflet from "leaflet";
  import { GeodesicLine } from "leaflet.geodesic";
  import "leaflet-edgebuffer";

  import { type ConnectionDirection } from "tauri-plugin-pcap-api";
  import { type Coordinate } from "tauri-plugin-ipgeo-api";
  import { CAPTURE_COLORS, CAPTURE_VARY_SIZE, lerp } from "$lib/utils";

  import { type MapArgs } from "$lib/page.svelte";
  import { fade } from "svelte/transition";

  let { capture, focused = $bindable(), children }: MapArgs = $props();

  let map: Map | null = $state(null);

  const arcs: Record<string, GeodesicLine> = {};
  const markers: Record<string, { mrk: Marker; count: number }> = {};

  const mapAction = (cont: HTMLDivElement) => {
    map = leaflet.map(cont, {
      preferCanvas: false,
      minZoom: 2,
      maxZoom: 15,
      zoomControl: false,
    });
    map.setView([25, 0], 2);
    map.attributionControl.remove();
    map.setMaxBounds(new LatLngBounds([-150, -300], [150, 400]));
    map.on("click", () => setFocused(null));

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

  const updateIcon = (key: string) => {
    if (!(key in markers)) return;

    const iconSize = focused === key ? 30 : 20;
    const iconAnchor = iconSize / 2;

    markers[key].mrk.setIcon(
      leaflet.divIcon({
        html: `<span>${markers[key].count}</span>`,
        className: focused === key ? "marker-icon-active" : "marker-icon",
        iconSize: [iconSize, iconSize],
        iconAnchor: [iconAnchor, iconAnchor],
      }),
    );
  };

  const calcWeight = (v: number) =>
    lerp(v, 0, capture?.maxThroughput ?? 1, 1.5, 6);
  const calcOpacity = (v: number) =>
    lerp(v, 0, capture?.maxThroughput ?? 1, 0.25, 1);

  const setFocused = (key: string | null) => {
    if (!capture) return;

    const old = focused;
    focused = key;

    if (focused) updateIcon(focused);
    if (old) updateIcon(old);
  };

  export const createMarker = (key: string, crd: Coordinate, count: number) => {
    if (!map) return;

    markers[key] = {
      count,
      mrk: leaflet.marker(crd).on("click", () => setFocused(key)),
    };

    // set icon after marker[key].count is set, markerIcon reads from that.
    updateIcon(key);

    markers[key].mrk.addTo(map);
  };

  export const updateMarker = (key: string, crd: Coordinate, count: number) => {
    if (!(key in markers)) {
      createMarker(key, crd, count);
      return;
    }

    // only update icon count when locCount updates.
    if (markers[key].count != count) {
      markers[key].count = count;
      updateIcon(key);
    }
  };

  export const removeMarker = (key: string) => {
    if (focused === key) setFocused(null);

    if (key in markers) {
      markers[key].mrk.remove();
      delete markers[key];
    }
  };

  export const createArc = (
    key: string,
    from: Coordinate,
    to: Coordinate,
    thr: number,
    dir: ConnectionDirection,
  ) => {
    if (!map) return;

    arcs[key] = new GeodesicLine([from, to], {
      steps: 5,
      // TODO: default styles other than basic blue?
      className: CAPTURE_COLORS ? dir : "",
      weight: CAPTURE_VARY_SIZE ? calcWeight(thr) : 2,
      opacity: CAPTURE_VARY_SIZE ? calcOpacity(thr) : 0.8,
    }).addTo(map);
  };

  export const updateArc = (
    key: string,
    from: Coordinate,
    to: Coordinate,
    thr: number,
    dir: ConnectionDirection,
  ) => {
    if (!(key in arcs)) {
      createArc(key, from, to, thr, dir);
      return;
    }

    const arc = arcs[key];

    if (CAPTURE_COLORS) {
      const svgElement = arc.getElement();
      if (svgElement) {
        svgElement.setAttribute("class", `leaflet-interactive ${dir}`);
      }
    }

    if (CAPTURE_VARY_SIZE) {
      arc.setStyle({
        weight: calcWeight(thr),
        opacity: calcOpacity(thr),
      });
    }
  };

  export const removeArc = (key: string) => {
    if (key in arcs) {
      arcs[key].remove();
    }

    delete arcs[key];
  };

  export const flyToPoint = (crd: Coordinate, zoom: number) => {
    if (map) map.flyTo(crd, lerp(zoom, 0, 1, 2, 13), { duration: 2 });
  };
</script>

<svelte:window on:resize={() => map?.invalidateSize()} />

<div
  in:fade={{ duration: 300 }}
  out:fade={{ duration: 200 }}
  class="absolute top-0 left-0 h-full w-full"
>
  <div class="join join-vertical absolute top-2 left-2 z-999 select-none">
    <button
      onclick={() => map?.zoomIn()}
      class="btn join-item btn-sm border-b-0 border-white text-xl font-bold"
      >+</button
    >
    <button
      onclick={() => map?.zoomOut()}
      class="btn join-item btn-sm border-t-0 border-white text-xl font-bold"
      >&#x2212;</button
    >
  </div>
  <div use:mapAction class="relative h-full w-full"></div>
  {@render children?.()}
</div>
