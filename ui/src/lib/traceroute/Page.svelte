<script lang="ts">
  import IpSearchBox from "$lib/IpSearchBox.svelte";
  import { Channel } from "@tauri-apps/api/core";
  import {
    database,
    netRawEnabled,
    runTraceroute,
    type Coordinate,
    type Hop,
    type Result,
    type TraceroutePreferences,
  } from "../../bindings";
  import Results from "./Results.svelte";

  const DEFAULT_PREFS: TraceroutePreferences = {
    maxRounds: 5,
    ip: "",
  };

  const resetPrefs = () => (prefs = DEFAULT_PREFS);

  let prefs: TraceroutePreferences = $state(DEFAULT_PREFS);

  // string => error message
  // number => loading round
  // null => inputting data
  // array => viewing results
  let pageState: Hop[] | string | number | null = $state(null);

  const search = async (input: Result<string, string> | null) => {
    if (input == null || input.status == "error") {
      pageState = input?.error ?? null;
      return;
    }

    pageState = 0;
    prefs.ip = input.data;
    const trace = await runTraceroute(
      prefs,
      new Channel((p) => (pageState = p)),
    );

    if (trace.status == "error") {
      pageState = trace.error;
      return;
    }

    pageState = trace.data;
  };

  let myLocation: Coordinate = $state({ lat: 0, lng: 0 });
  database.myLocation().then((l) => {
    if (l) myLocation = l.crd;
  });
</script>

<div class="flex h-full w-full grow flex-col">
  {#await netRawEnabled() then enabled}
    {#if !enabled}
      <p>Privileges are not available to access traceroute</p>
    {:else if pageState != null && typeof pageState == "number"}
      <div
        class="flex grow flex-col items-center justify-items-center space-y-3"
      >
        <span class="loading loading-spinner loading-2xl"></span>
        <p>Round: {pageState}</p>
      </div>
    {:else if pageState != null && typeof pageState == "string"}
      <p>Error: {pageState}</p>
      <button class="btn btn-primary" onclick={() => (pageState = null)}
        >Ok</button
      >
    {:else if pageState != null && typeof pageState == "object"}
      <Results
        hops={pageState}
        {myLocation}
        ip={prefs.ip}
        close={() => (pageState = null)}
      />
    {:else}
      <fieldset
        class="fieldset bg-base-200 border-base-300 rounded-box w-xs self-center border p-4"
      >
        <legend class="fieldset-legend">Run a Traceroute</legend>

        <label class="label" for="maxRounds">Rounds</label>
        <input
          id="maxRounds"
          type="number"
          min="1"
          max="100"
          bind:value={prefs.maxRounds}
          class="input input-sm"
        />

        <label class="label" for="ipsearchbox">IP Address or Domain</label>
        <IpSearchBox {search} />
      </fieldset>
    {/if}
  {/await}
</div>
