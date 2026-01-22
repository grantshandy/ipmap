<script lang="ts">
  import "@openglobus/og/styles";
  import {
    wgs84,
    Bing,
    Entity,
    Globe,
    GlobusRgbTerrain,
    LonLat,
    Vector,
    Renderer,
  } from "@openglobus/og";
  import { type MapArgs } from "$lib/page.svelte";
  import {
    getDashedPath,
    getPath,
    DASH_TO_GAP_RATIO,
    NUMBER_OF_DASHES,
    type ArcRecord,
  } from "$lib/3d-arc";
  import { asset } from "$app/paths";
  import { lerp, CAPTURE_COLORS, CAPTURE_VARY_SIZE } from "$lib/utils";
  import { fade } from "svelte/transition";

  import { type ConnectionDirection } from "tauri-plugin-pcap-api";
  import { type Coordinate } from "tauri-plugin-ipgeo-api";

  // TODO: tweak
  const ZOOM_SPEED = 0.05;
  const MIN_DELTA = 0.1;
  const MAX_DELTA = 1000_0000000;
  const ZOOM_FACTOR = 0.05;

  let { capture, focused = $bindable() }: MapArgs = $props();

  let globe: Globe | null = $state(null);
  let arcRecords: Record<string, ArcRecord> = {};
  let markerRecords: Record<string, Entity> = {};
  let zoomState = 0;

  const markers = new Vector("points");
  const arcs = new Vector("arcs");

  const addGlobe = (target: HTMLElement) => {
    globe = new Globe({
      target,
      name: "Earth",
      terrain: new GlobusRgbTerrain(),
      atmosphereEnabled: true,
      layers: [new Bing(null), markers, arcs],
      controls: [],
      attributionContainer: document.createElement("div"),
      navigation: {
        mode: "lockNorth",
        disableRotation: true,
      },
    });

    globe.start();

    globe.planet?.renderer!.events.on("draw", (e: Renderer) => {
      if (zoomState === 0 || !globe || !globe.planet.camera) return;

      const pos = globe.planet.getCartesianFromPixelTerrain(e.getCenter());
      if (!pos) return;

      const cam = globe.planet.camera;
      let distance = cam.eye.distance(pos);

      let delta = Math.min(
        MAX_DELTA,
        Math.max(MIN_DELTA, ZOOM_FACTOR * Math.pow(distance, 0.9)),
      );

      cam.eye.addA(cam.getForward().scale(zoomState * delta));
      cam.checkTerrainCollision();
      cam.update();

      if (zoomState > 0) {
        zoomState -= ZOOM_SPEED;
      } else {
        zoomState += ZOOM_SPEED;
      }

      if (Math.abs(zoomState) < 0.1) {
        zoomState = 0;
      }

      console.log({ zoomState, distance, delta });
    });

    return {
      destroy: () => globe?.destroy(),
    };
  };

  const calcOpacity = (thr: number) =>
    lerp(thr, 0, capture?.maxThroughput ?? 1, 0.25, 1);
  const calcThickness = (thr: number) =>
    lerp(thr, 0, capture?.maxThroughput ?? 1, 3, 6);
  const calcColor = (clr: ConnectionDirection) =>
    clr === "up" ? "#c01c28" : clr === "down" ? "#26a269" : "#cd9309";

  export const createMarker = (
    key: string,
    crd: Coordinate,
    _count: number,
  ) => {
    if (!globe) return;

    const coords = new LonLat(crd.lng, crd.lat);

    markerRecords[key] = new Entity({
      name: key,
      lonlat: coords,
      billboard: {
        src: asset("/marker.png"),
        size: [18, 32],
        offset: [0, 16],
        alignedAxis: wgs84.lonLatToCartesian(coords).normalize(),
      },
    }).addTo(markers);
  };

  export const updateMarker = (key: string, crd: Coordinate, count: number) => {
    if (!globe) return;

    if (!(key in markerRecords)) {
      createMarker(key, crd, count);
      return;
    }
  };

  export const removeMarker = (key: string) => {
    if (!(key in markerRecords)) return;

    markerRecords[key].remove();
    delete markerRecords[key];
    markers.update();
  };

  export const createArc = (
    key: string,
    from: Coordinate,
    to: Coordinate,
    thr: number,
    dir: ConnectionDirection,
  ) => {
    if (!globe) return;

    const fullPath = getPath(globe.planet.ellipsoid, from, to);

    const dashedPath = getDashedPath(
      fullPath,
      NUMBER_OF_DASHES,
      DASH_TO_GAP_RATIO,
      0,
    );

    arcRecords[key] = {
      arc: new Entity({
        polyline: {
          path3v: CAPTURE_COLORS ? dashedPath : [fullPath],
          color: CAPTURE_COLORS ? calcColor(dir) : "white",
          thickness: CAPTURE_VARY_SIZE ? calcThickness(thr) : 4,
          opacity: CAPTURE_VARY_SIZE ? calcOpacity(thr) : 0.65,
          isClosed: false,
        },
      }).addTo(arcs),
      animIndex: 0,
      fullPath,
      direction: dir,
    };
  };

  export const updateArc = (
    key: string,
    from: Coordinate,
    to: Coordinate,
    thr: number,
    dir: ConnectionDirection,
  ) => {
    if (!capture) return;

    if (!(key in arcRecords)) {
      createArc(key, from, to, thr, dir);
      return;
    }

    arcRecords[key].direction = dir;

    const polyline = arcRecords[key].arc.polyline;
    if (!polyline) return;

    if (CAPTURE_VARY_SIZE) {
      polyline.setThickness(calcThickness(thr));
      polyline.setOpacity(calcOpacity(thr));
    }

    if (CAPTURE_COLORS) {
      polyline.setColorHTML(calcColor(dir));
    }
  };

  export const removeArc = (key: string) => {
    if (!(key in arcRecords)) return;

    arcRecords[key].arc.remove();
    arcs.update();
    delete arcRecords[key];
  };

  export const flyToPoint = (crd: Coordinate, zoom: number): void => {
    if (!globe?.planet) return;

    globe?.planet.camera.flyLonLat(new LonLat(crd.lng, crd.lat));
  };

  export const zoomIn = (): void => {
    zoomState = 1;
  };

  export const zoomOut = (): void => {
    zoomState = -1;
  };
</script>

<div
  in:fade={{ duration: 300 }}
  out:fade={{ duration: 200 }}
  class="absolute top-0 left-0 h-full w-full"
>
  <div
    use:addGlobe
    class="overflow-none relative h-full min-h-0 w-full grow"
  ></div>
</div>
