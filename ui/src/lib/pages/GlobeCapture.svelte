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
    animateLine,
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

  const onUpdate = (crd: string, loc: CaptureLocation) => {
    if (!capture || !(crd in locations)) return;

    const { opacity, weight } = calculateWeights(
      loc.thr,
      capture.maxThroughput,
    );

    locations[crd].ent.polyline?.setOpacity(opacity);
    locations[crd].ent.polyline?.setThickness(weight);

    if (!(crd in capture.connections)) return;

    if (locations[crd].direction != capture.connections[crd].dir) {
      locations[crd].direction = capture.connections[crd].dir;
      console.log("changing direction");
      locations[crd].colors = directionArcColors(locations[crd].direction);
    }
  };

  const onStopping = () => {
    for (const record of Object.values(locations)) {
      record.ent.remove();
    }

    locations = {};
    arcs.setEntities([]);
  };

  const onIpChanged = (crd: string, record: CaptureLocation | null) => {
    if (!capture || !globe) return;

    // delete this arc
    if (record == null) {
      if (crd in locations) locations[crd].ent.remove();
      delete locations[crd];
      return;
    }

    // add a new arc
    if (Object.keys(record.ips).length == 1) {
      const { path, colors } = getPath(
        globe.planet.ellipsoid,
        myLocation,
        record.crd,
        record.dir,
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
        direction: record.dir,
      };

      arcs.add(ent);
    }
  };

  const startCapture = () => {
    capture = pcap.startCapture(onUpdate, onIpChanged, onStopping);
  };

  const stopCapture = async () => {
    await pcap.stopCapture();
    capture = null;
  };

  $effect(() => {
    // if (!globe) return;
    // globe.planet.renderer?.handler.defaultClock.setInterval(10, () => {
    //   for (const loc of Object.values(locations)) {
    //     animateLine(loc);
    //   }
    // });
  });
</script>

<GlobeMap bind:globe layers={[arcs]}>
  <div class="absolute top-2 right-2 z-[999]">
    <CaptureStart {pcap} {startCapture} {stopCapture} />
  </div>
</GlobeMap>
