<script lang="ts">
  import MapView from "./MapView.svelte";
  import LocationInfoView from "./LocationInfoView.svelte";
  import IpView from "./IpView.svelte";

  import { onDestroy } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { layerGroup, marker, type Map } from "leaflet";
  import { GeodesicLine } from "leaflet.geodesic";

  import { database } from "../utils/database";
  import { mkIcon, type IpLocation } from "../map";
  import {
    type ConnectionInfo,
    type Coordinate,
    type Device,
    type ThreadID,
    capture,
    geoip,
  } from "../bindings";

  const POLL_MS = 250;

  let map: Map;

  const markerLayer = layerGroup();
  let markerLayerVisible = false;
  $: if (map) {
    markerLayer.addTo(map);
    markerLayerVisible = true;
  }

  const arcLayer = layerGroup();
  let arcLayerVisible = false;
  $: if (map) {
    arcLayer.addTo(map);
    arcLayerVisible = true;
  }

  let device: Device | null = null;
  let capturing: { id: ThreadID; unlisten: UnlistenFn } | null = null;

  const toggleCapturing = async () => {
    if (!device) return;

    if (capturing) {
      setArcState([]);
      await capture.stopCapturing(capturing.id);
      capturing = null;
    } else {
      const unlisten = await capture.onNewConnection((ip) => {
        if (!capturing) unlisten();
        addIp(ip);
      });

      capturing = {
        id: await capture.startCapturing(device),
        unlisten,
      };

      currentConnectionLoop();
    }
  };

  const cleanup = () => {
    if (capturing) {
      console.log("stopping capture of " + capturing.id);
      capture.stopCapturing(capturing.id);
      capturing.unlisten();
      capturing = null;
    }
  };
  onDestroy(cleanup);
  window.onbeforeunload = cleanup;

  const currentConnectionLoop = () => {
    if (!capturing) {
      setArcState([]);
      return;
    }

    capture.currentConnections().then(setArcState);
    setTimeout(currentConnectionLoop, POLL_MS);
  };

  // auto-updating location from database
  const myLocation = marker([0, 0], { icon: mkIcon(null, false) }).addTo(
    markerLayer,
  );
  $: if ($database)
    geoip.myLocation().then((c) => {
      myLocation.setLatLng(c);
      for (const state of Object.values(arcState)) {
        state.arc.setLatLngs([c, state.to]);
      }
    });

  // current state of arcs (connections)
  let arcState: {
    [id: string]: { info: ConnectionInfo; arc: GeodesicLine; to: Coordinate };
  } = {};

  const setArcState = async (newState: ConnectionInfo[]) => {
    const newStates: { [id: string]: ConnectionInfo } = {};
    for (const i of newState) newStates[i.ip] = i;

    // remove or change previously added arcs
    for (const prevState of Object.values(arcState)) {
      const ip = prevState.info.ip;

      if (newStates[ip]) {
        // update direction if needed
        if (prevState.info.direction != newStates[ip].direction) {
          prevState.arc.options.className = newStates[ip].direction as string;
        }
      } else {
        // outdated arcs that no longer exist should be removed
        arcState[ip].arc.remove();
        delete arcState[ip];
      }
    }

    // add arcs that don't already exist
    for (const newState of Object.values(newStates)) {
      // discard already existing connections
      if (arcState[newState.ip]) continue;

      geoip.lookupIp(newState.ip).then((to) => {
        if (!to) return;

        arcState[newState.ip] = {
          arc: new GeodesicLine([myLocation.getLatLng(), to], {
            weight: 2,
            steps: 3,
            opacity: 0.5,
            className: newState.direction,
          }).addTo(arcLayer),
          info: newState,
          to,
        };
      });
    }
  };

  type LocationKey = string;
  const locationKey = (c: Coordinate): LocationKey => `${c.lat}${c.lng}`;

  let selection: IpLocation | null = null;
  let locations: {
    [id: LocationKey]: IpLocation;
  } = {};
  let ips: Set<string> = new Set();

  const addIp = async (ip: string) => {
    if (ips.has(ip)) return;
    ips.add(ip);

    const coord = await geoip.lookupIp(ip);

    if (!coord) {
      console.warn(ip + " not found in database");
      return;
    }

    const key = locationKey(coord);
    const loc = locations[key];

    if (loc) {
      loc.ips.add(ip);
      loc.marker.setIcon(mkIcon(loc.ips.size, false));
    } else {
      locations[key] = {
        coord,
        ips: new Set([ip]),
        marker: marker(coord, { icon: mkIcon(1, false) })
          .on("click", () => markerOnClick(key))
          .addTo(markerLayer),
      };
    }
  };

  const markerOnClick = (key: LocationKey) => {
    // wait for animation with timeout
    if (selection) {
      selection.marker
        .setIcon(mkIcon(selection.ips.size, false))
        .setZIndexOffset(50);
    }

    if (selection && locationKey(selection.coord) == key) {
      selection = null;
      return;
    }

    selection = locations[key];
    if (!selection) return;

    if (map.getZoom() < 5) {
      map.flyTo(selection.coord, 5);
    } else {
      map.panTo(selection.coord);
    }

    selection.marker
      .setIcon(mkIcon(selection.ips.size, true))
      .setZIndexOffset(1000);
  };
</script>

<div class="flex grow flex-col space-y-3">
  <hr class="select-bordered" />

  <div class="flex select-none">
    <div class="flex grow space-x-3">
      <div class="indicator w-1/3">
        {#if device == null}
          <span class="badge indicator-item badge-primary badge-sm"></span>
        {/if}
        <select
          bind:value={device}
          disabled={capturing != null}
          class="select select-bordered select-sm w-full"
        >
          <option disabled selected value={null}>Select Network Device</option>
          {#await capture.listDevices() then devices}
            {#each devices as device}
              <option value={device}>
                {device.desc ?? `${device.name} (No Description)`}
                {device.prefered ? " (Default)" : ""}
              </option>
            {/each}
          {/await}
        </select>
      </div>

      <button
        on:click={toggleCapturing}
        disabled={!device}
        class="btn btn-primary btn-sm"
        class:btn-active={capturing}
      >
        {capturing ? "Stop" : "Start"} Capturing
      </button>
    </div>

    <div class="flex space-x-3">
      <button
        on:click={() => {
          if (markerLayerVisible) {
            markerLayer.remove();
          } else {
            markerLayer.addTo(map);
          }

          markerLayerVisible = !markerLayerVisible;
        }}
        class="btn btn-square btn-ghost btn-sm p-2"
        class:btn-active={markerLayerVisible}
      >
        <div class="marker-icon drop-shadow-none"></div>
      </button>
      <button
        on:click={() => {
          if (arcLayerVisible) {
            arcLayer.remove();
          } else {
            arcLayer.addTo(map);
          }

          arcLayerVisible = !arcLayerVisible;
        }}
        class="btn btn-square btn-ghost btn-sm p-2"
        class:btn-active={arcLayerVisible}
      >
        <div
          class="h-full w-full rounded-full border-[0.15rem] border-dotted border-warning"
        />
      </button>
    </div>
  </div>

  <div class="flex grow space-x-3">
    <MapView bind:map>
      <div
        class="absolute bottom-0 left-0 z-30 flex select-none items-center rounded-tr-box bg-base-100 pr-1.5 pt-0.5 text-xs"
      >
        <div class="color-indicator bg-success" />
        <span>Incoming</span>
        <div class="color-indicator bg-error" />
        <span>Outgoing</span>
        <div class="color-indicator bg-warning" />
        <span>Mixed</span>
      </div>

      {#if selection}
        <div class="map-info-panel overflow-y-auto">
          <div
            class="space-y-3 rounded-box border border-base-300 bg-base-100 px-3 py-2"
          >
            <h2 class="select-none font-semibold">Location Information</h2>
            <LocationInfoView coord={selection.coord} />
          </div>
          {#each selection.ips as ip, i}
            <div
              class="space-y-3 rounded-box border border-base-300 bg-base-100 px-3 py-2"
            >
              <h3 class="font-semibold">
                {i + 1}: <span class="italic">{ip}</span>
              </h3>
              <IpView {ip} />
            </div>
          {/each}
        </div>
      {/if}
    </MapView>
  </div>
</div>

<style lang="postcss">
  .color-indicator {
    @apply ml-1.5 mr-0.5 h-3 w-3 rounded-full;
  }
</style>
