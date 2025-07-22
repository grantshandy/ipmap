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
    directionColorString,
    getPath,
    type LocationRecord,
  } from "$lib/globus-utils";
  import { Entity, Vector, type Globe } from "@openglobus/og";
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

    const ent = new Entity({
      polyline: {
        path3v: [getPath(globe.planet.ellipsoid, myLocation, loc)],
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
