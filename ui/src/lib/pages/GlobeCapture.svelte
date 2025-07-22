<script lang="ts">
  import CaptureStart from "$lib/components/CaptureStart.svelte";
  import GlobeMap from "$lib/components/Globe.svelte";

  import {
    CaptureSession,
    database,
    type CaptureLocation,
    type Coordinate,
    type Pcap,
  } from "$lib/bindings";
  import {
    calculateOpacity,
    calculateWeight,
    directionArcColors,
    getPath,
    type LocationRecord,
  } from "$lib/globus-utils";
  import { Entity, Vector, type Globe } from "@openglobus/og";
  import { onDestroy } from "svelte";
  import { calculateWeights } from "$lib/leaflet-utils";

  const { pcap }: { pcap: Pcap } = $props();

  onDestroy(() => {
    pcap.stopCapture();
  });

  let globe: Globe | null = $state(null);
  let capture: CaptureSession | null = $state(null);
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

    const { path, colors } = getPath(
      globe.planet.ellipsoid,
      myLocation,
      loc,
      pcap.capture.maxThroughput,
    );

    const ent = new Entity({
      polyline: {
        path3v: [path],
        pathColors: [colors],
        thickness: 2,
        isClosed: false,
      },
    });

    locations[crd] = {
      ent,
      animIndex: 0,
      colors,
      direction: loc.dir,
    };

    arcs.add(ent);
  };

  const locationRemoved = (crd: string) => {
    if (crd in locations) {
      locations[crd].ent.remove();
      delete locations[crd];
    }
  };

  const update = async (crd: string, loc: CaptureLocation) => {
    if (pcap.capture && crd in locations) {
      locations[crd].ent.polyline?.setOpacity(
        calculateOpacity(loc.thr, pcap.capture.maxThroughput),
      );
      locations[crd].ent.polyline?.setThickness(
        calculateWeight(loc.thr, pcap.capture.maxThroughput),
      );

      // if (!(crd in pcap.capture.connections)) return;

      // if (locations[crd].direction != capture.connections[crd].dir) {
      //   locations[crd].direction = capture.connections[crd].dir;
      //   console.log("changing direction");
      //   locations[crd].colors = directionArcColors(locations[crd].direction);
      // }
    }
  };

  const stopping = () => {
    locations = {};
    arcs.setEntities([]);
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
