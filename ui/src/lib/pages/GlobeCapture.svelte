<script lang="ts">
  import CaptureStart from "$lib/components/CaptureStart.svelte";
  import GlobeMap from "$lib/components/Globe.svelte";

  import {
    database,
    lerp,
    type CaptureLocation,
    type ConnectionDirection,
    type Coordinate,
    type Pcap,
  } from "$lib/bindings";
  import {
    getDashedPath,
    getPath,
    DASH_TO_GAP_RATIO,
    NUMBER_OF_DASHES,
    OSCILATION_RANGE,
    type LocationRecord,
  } from "$lib/3d-arc";
  import { Entity, Vector, type Globe } from "@openglobus/og";
  import { onDestroy } from "svelte";

  const { pcap }: { pcap: Pcap } = $props();

  onDestroy(() => pcap.stopCapture());

  let globe: Globe | null = $state(null);
  let locations: Record<string, LocationRecord> = {};

  const arcs = new Vector("Collection", {
    entities: [],
    zIndex: 1,
  });

  let myLocation: Coordinate = { lat: 0, lng: 0 };
  database.myLocation().then((l) => {
    if (l.status == "ok") {
      myLocation = l.data.crd;
    } else {
      console.warn(l.error);
    }
  });

  const calcOpacity = (thr: number) =>
    lerp(thr, 0, pcap.capture?.maxThroughput ?? 0, 0.25, 1);
  const calcThickness = (thr: number) =>
    lerp(thr, 0, pcap.capture?.maxThroughput ?? 0, 3, 6);
  const calcColor = (clr: ConnectionDirection) =>
    clr === "up" ? "#c01c28" : clr === "down" ? "#26a269" : "#cd9309";

  const locationAdded = (crd: string, loc: CaptureLocation) => {
    if (!globe || !pcap.capture || !(crd in pcap.capture.connections)) return;

    const fullPath = getPath(globe.planet.ellipsoid, myLocation, loc);

    const dashedPath = getDashedPath(
      fullPath,
      NUMBER_OF_DASHES,
      DASH_TO_GAP_RATIO,
      0,
    );

    locations[crd] = {
      ent: new Entity({
        polyline: {
          path3v: dashedPath,
          color: calcColor(loc.dir),
          thickness: calcThickness(loc.thr),
          opacity: calcOpacity(loc.thr),
          isClosed: false,
        },
      }).addTo(arcs),
      animIndex: 0,
      fullPath,
    };
  };

  const locationRemoved = (crd: string) => {
    if (crd in locations) {
      locations[crd].ent.remove();
      arcs.update();
      delete locations[crd];
    }
  };

  const update = async (crd: string, loc: CaptureLocation) => {
    if (!pcap.capture || !(crd in locations)) return;

    const polyline = locations[crd].ent.polyline;
    if (!polyline) return;

    polyline.setOpacity(calcOpacity(loc.thr));
    polyline.setThickness(calcThickness(loc.thr));
    polyline.setColorHTML(calcColor(loc.dir));
  };

  const stopping = () => {
    locations = {};
    arcs.setEntities([]);
    arcs.update();
  };

  const onGlobeInit = (globe: Globe) => {
    globe.planet.renderer?.handler.defaultClock.setInterval(10, () => {
      if (!pcap.capture) return;

      for (const crd in locations) {
        if (!(crd in pcap.capture.connections)) continue;

        const locRecord = locations[crd];
        const { ent, fullPath } = locRecord;
        const { dir } = pcap.capture.connections[crd];

        locRecord.animIndex += 1;

        let offset;

        // Determine the direction of movement
        if (dir === "mixed") {
          // Use a triangular wave function to create a back-and-forth motion
          const doubledRange = OSCILATION_RANGE * 2;
          offset = Math.abs(
            (locRecord.animIndex % doubledRange) - OSCILATION_RANGE,
          );
        } else {
          offset = dir === "up" ? locRecord.animIndex : -locRecord.animIndex;
        }

        ent.polyline?.setPath3v(
          getDashedPath(fullPath, NUMBER_OF_DASHES, DASH_TO_GAP_RATIO, offset),
        );
      }
    });
  };
</script>

<GlobeMap bind:globe layers={[arcs]} {onGlobeInit}>
  <div class="absolute top-2 right-2 z-[999]">
    <CaptureStart
      {pcap}
      callbacks={{
        locationAdded,
        locationRemoved,
        update,
        stopping,
      }}
    />
  </div>
</GlobeMap>
