<script lang="ts">
  import MapView from "../components/MapView.svelte";
  import IpAddrInput from "../components/IpAddrInput.svelte";
  import IpLocationView from "../components/IpView.svelte";

  import { marker, Map } from "leaflet";

  import { geoip, type Coordinate } from "../bindings";
  import { mkIcon, type IpLocation } from "../map";
  import LocationName from "../components/LocationInfoView.svelte";

  let map: Map;
  let error: string | null = null;
  let selection: IpLocation | null = null;

  const validateAndSearch = async (ip: string) => {
    if (!map) return;

    const coord = await geoip.lookupIp(ip);
    if (!coord) {
      error = "IP Not Found in Database";
      setSearchIp(null);
      return;
    }

    error = null;
    setSearchIp(ip, coord), 300;
  };

  const setSearchIp = (ip: string | null, coord?: Coordinate) => {
    if (selection) {
      selection.marker.remove();
      selection = null;
    }

    if (!coord || !map || !ip) return;

    selection = {
      ips: new Set([ip]),
      coord,
      marker: marker(coord, { icon: mkIcon(null, true) }).addTo(map),
    };

    if (map.getZoom() == 7) {
      map.panTo(coord);
    } else {
      map.flyTo(coord, 7);
    }
  };

  $: if (error && selection) {
    selection?.marker.remove();
    selection = null;
  }
</script>

<div class="flex grow space-x-2">
  <MapView bind:map>
    <div class="map-info-panel">
      <IpAddrInput bind:error onSearch={validateAndSearch} />
      {#if error}
        <p class="grow select-none p-2 text-sm font-bold italic text-error">
          {error}
        </p>
      {/if}
      {#if selection}
        <div
          class="border-base-300r space-y-3 rounded-box border bg-base-100 px-3 py-2"
        >
          <LocationName coord={selection.coord} />
          {#each selection.ips as ip}
            <IpLocationView {ip} />
          {/each}
        </div>
      {/if}
    </div>
  </MapView>
</div>
