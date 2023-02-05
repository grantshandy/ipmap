<script lang="ts">
  import "../tailwind.css";

  import Map from "../Map.svelte";
  import type { Connection } from "../geolocate";
  import { Vector } from "ol/source";
  import { Feature } from "ol";
  import { Point } from "ol/geom";

  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api";
  import { fromLonLat } from "ol/proj";

  let mapSource: Vector = new Vector();

  const queue: Record<string, number> = {}; // ips currently being fetched from the API/cache.
  const connections: Record<string, Connection> = {}; // ips logged, fetched, and stored

  listen("connection", (event: any) => {
    let ip: string = event.payload as string;

    if (!ip) {
      return;
    }

    if (connections[ip]) {
      connections[ip].count += 1;
      return;
    }

    if (queue[ip]) {
      queue[ip] += 1;
      return;
    } else {
      queue[ip] = 1;
    }

    invoke("get_cache", { query: ip }).then((resp) => {
      const cacheConn = resp as Connection;

      if (!cacheConn) {
        invoke("fetch_connection", { query: ip }).then((resp) => {
          const newConn = resp as Connection;

          if (newConn) {
            newConn.count = queue[ip];
            delete queue[ip];
            connections[newConn.ip] = newConn;
            addToMap(newConn);
          } else {
            console.log("no new conn?");
          }
        });
      } else {
        cacheConn.count = queue[ip];
        delete queue[ip];
        connections[cacheConn.ip] = cacheConn;
        addToMap(cacheConn);
      }
    });
  });

  function addToMap(conn: Connection) {
    mapSource.addFeature(
      new Feature(new Point(fromLonLat([conn.longitude, conn.latitude])))
    );
  }
</script>

<div class="w-screen h-screen flex">
  <Map bind:source={mapSource} />
  <!-- div class="p-3 text-center">
    <h1 class="text-xl font-semibold">Connected IPs</h1>
  </div -->
</div>
