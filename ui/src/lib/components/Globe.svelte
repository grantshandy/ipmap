<script lang="ts">
  import "@openglobus/og/styles";
  import {
    Bing,
    control,
    Entity,
    Globe,
    GlobusRgbTerrain,
    LonLat,
    Vec3,
    Vector,
    wgs84,
  } from "@openglobus/og";
  import { type MapArgs } from "$lib/page.svelte";
  import {
    getDashedPath,
    getPath,
    DASH_TO_GAP_RATIO,
    NUMBER_OF_DASHES,
    OSCILATION_RANGE,
    type ArcRecord,
  } from "$lib/3d-arc";

  import { asset } from "$app/paths";
  import { lerp, CAPTURE_COLORS, CAPTURE_VARY_SIZE } from "$lib/utils";

  import { fade } from "svelte/transition";

  import { type ConnectionDirection } from "tauri-plugin-pcap-api";
  import { type Coordinate } from "tauri-plugin-ipgeo-api";

  let { capture, focused = $bindable() }: MapArgs = $props();

  let globe: Globe | null = $state(null);
  let arcRecords: Record<string, ArcRecord> = {};
  let markerRecords: Record<string, Entity> = {};

  const markers = new Vector("points");
  const arcs = new Vector("arcs");

  const addGlobe = (target: HTMLElement) => {
    globe = new Globe({
      target,
      name: "Earth",
      terrain: new GlobusRgbTerrain(),
      atmosphereEnabled: true,
      layers: [new Bing(null), markers, arcs],
      controls: [new control.MouseNavigation()],
      attributionContainer: document.createElement("div"),
    });

    globe.start();

    globe.planet.renderer?.handler.defaultClock.setInterval(10, () => {
      if (capture != null && !CAPTURE_COLORS) return;

      for (const crd in arcRecords) {
        const locRecord = arcRecords[crd];
        const { arc: ent, fullPath, direction } = locRecord;

        locRecord.animIndex += 1;

        let offset;

        // Determine the direction of movement
        if (direction === "mixed") {
          // Use a triangular wave function to create a back-and-forth motion
          const doubledRange = OSCILATION_RANGE * 2;
          offset = Math.abs(
            (locRecord.animIndex % doubledRange) - OSCILATION_RANGE,
          );
        } else {
          offset =
            direction === "up" ? locRecord.animIndex : -locRecord.animIndex;
        }

        ent.polyline?.setPath3v(
          getDashedPath(fullPath, NUMBER_OF_DASHES, DASH_TO_GAP_RATIO, offset),
        );
      }
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

    globe?.planet.camera.flyLonLat(
      new LonLat(crd.lng, crd.lat, lerp(zoom, 0, 1, 10_000_000, 250_000)),
    );
  };

  export const zoomIn = (): void => {
    // TODO: implement
  };

  export const zoomOut = (): void => {
    // TODO: implement
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
