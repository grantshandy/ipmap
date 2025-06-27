<script lang="ts">
  import MapView from "./Map.svelte";

  import { database } from "../bindings";
  import type { Map } from "leaflet";

  let map: Map | null = $state(null);
</script>

<div class="flex grow">
  <MapView bind:map>
    <form
      class="join join-horizontal bg-base-300 rounded-box absolute top-2 right-2 z-[999] border select-none"
      onsubmit={search}
    >
      <input
        type="text"
        class="input input-sm join-item"
        placeholder="IP Address"
        oninput={() => {
          ip = null;
          result = null;
        }}
        class:input-error={trimmedInput.length != 0 && !validInput}
        bind:value={input}
      />
      <button
        class="btn btn-sm btn-primary join-item"
        disabled={!validInput || ip != null}
        type="submit">Search</button
      >
    </form>
  </MapView>
</div>
