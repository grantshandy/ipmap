<script lang="ts">
  import CaptureStart from "$lib/components/CaptureStart.svelte";
  import GlobeMap from "$lib/components/Globe.svelte";

  import {
    database,
    type CaptureLocation,
    type Coordinate,
    type Pcap,
  } from "$lib/bindings";
  import {
    calculateOpacity,
    calculateWeight,
    DASH_TO_GAP_RATIO,
    directionColorString,
    getDashedPath,
    getPath,
    NUMBER_OF_DASHES,
    OSCILATION_RANGE,
    type LocationRecord,
  } from "$lib/globus-utils";
  import { Entity, Vec3, Vector, type Globe } from "@openglobus/og";
  import { onDestroy } from "svelte";

  const { pcap }: { pcap: Pcap } = $props();

  onDestroy(() => {
    pcap.stopCapture();
  });

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

  const locationAdded = (crd: string, loc: CaptureLocation) => {
    if (!globe || !pcap.capture || !(crd in pcap.capture.connections)) return;

    const fullPath = getPath(globe.planet.ellipsoid, myLocation, loc);

    const dashedPath = getDashedPath(
      fullPath,
      NUMBER_OF_DASHES,
      DASH_TO_GAP_RATIO,
      0,
    );

    const ent = new Entity({
      polyline: {
        path3v: dashedPath,
        color: directionColorString(loc.dir),
        thickness: calculateWeight(loc.thr, pcap.capture.maxThroughput),
        opacity: calculateOpacity(loc.thr, pcap.capture.maxThroughput),
        isClosed: false,
      },
    });

    locations[crd] = {
      ent,
      animIndex: 0,
      direction: loc.dir,
      fullPath,
    };

    arcs.add(ent);
  };

  const locationRemoved = (crd: string) => {
    if (crd in locations) {
      locations[crd].ent.remove();
      arcs.update();
      delete locations[crd];
    }
  };

  const update = async (crd: string, loc: CaptureLocation) => {
    if (pcap.capture && crd in locations) {
      const polyline = locations[crd].ent.polyline;
      if (!polyline) return;

      locations[crd].direction = loc.dir;

      polyline.setOpacity(
        calculateOpacity(loc.thr, pcap.capture.maxThroughput),
      );
      polyline.setThickness(
        calculateWeight(loc.thr, pcap.capture.maxThroughput),
      );
      polyline.setColorHTML(directionColorString(loc.dir));
    }
  };

  const stopping = () => {
    locations = {};
    arcs.setEntities([]);
    arcs.update();
  };

  $effect(() => {
    if (globe) {
      globe.planet.renderer?.handler.defaultClock.setInterval(10, () => {
        for (const crd in locations) {
          const locRecord = locations[crd];
          const { ent, fullPath, direction } = locRecord;

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
            getDashedPath(
              fullPath,
              NUMBER_OF_DASHES,
              DASH_TO_GAP_RATIO,
              offset,
            ),
          );
        }
      });
    }
  });
</script>

<GlobeMap bind:globe layers={[arcs]}>
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
