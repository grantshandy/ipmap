<script lang="ts">
  import IpSearchBox from "$lib/components/IpSearchBox.svelte";
  import TraceMap from "$lib/components/TraceMap.svelte";
  import ErrorScreen from "$lib/components/ErrorScreen.svelte";

  import { Channel } from "@tauri-apps/api/core";
  import {
    database,
    traceroute,
    isIpcError,
    type Coordinate,
    type Hop,
    type Result,
    type TracerouteParams,
    type IpcError,
  } from "$lib/bindings";
  import type { Action } from "svelte/action";

  const MAX_MAX_ROUNDS: number = 200;

  let prefs: TracerouteParams = $state({
    maxRounds: 5,
    ip: "",
  });

  // null => inputting data
  // number => loading round
  // Hop[] => viewing results
  // Error => error message
  let pageState: null | number | Hop[] | IpcError = $state(null);

  const search = async (input: Result<string, string> | null) => {
    if (input == null || input.status == "error") {
      pageState = input?.error ? { t: "Runtime", c: input.error } : null;
      return;
    }

    pageState = 0;
    prefs.ip = input.data;
    const res = await traceroute.run(
      prefs,
      new Channel((p) => (pageState = p)),
    );
    pageState = res.status == "ok" ? res.data : res.error;
  };

  let myLocation: Coordinate = $state({ lat: 0, lng: 0 });
  database.myLocation().then((l) => {
    if (l) myLocation = l.crd;
  });

  let formInvalid = $derived(
    prefs.maxRounds < 1 || prefs.maxRounds > MAX_MAX_ROUNDS,
  );

  const loadingBar: Action = (elem) => {
    $effect(() => {
      if (pageState != null && typeof pageState == "number")
        elem.style.width = `${(pageState / prefs.maxRounds) * 100}%`;
    });
  };
</script>

<div class="flex h-full w-full grow flex-col">
  {#await traceroute.enabled() then enabled}
    {#if enabled.status == "error"}
      <ErrorScreen error={enabled.error} />
    {:else if isIpcError(pageState)}
      <ErrorScreen bind:error={pageState} exitable={true} />
    {:else if typeof pageState == "number"}
      {@render traceLoading()}
    {:else if Array.isArray(pageState)}
      <TraceMap
        hops={pageState}
        {myLocation}
        ip={prefs.ip}
        close={() => (pageState = null)}
      />
    {:else}
      {@render tracerouteForm()}
    {/if}
  {/await}
</div>

{#snippet traceLoading()}
  <div class="flex grow items-center justify-center select-none">
    <div class="space-y-3 text-center">
      <p>Tracing...</p>

      <div class="progress w-56 overflow-hidden rounded-full">
        <div
          class="h-full rounded-r-full bg-white transition-all duration-500 ease-in-out"
          use:loadingBar
        ></div>
      </div>
    </div>
  </div>
{/snippet}

{#snippet tracerouteForm()}
  <div class="flex grow items-center justify-center">
    <fieldset
      class="fieldset bg-base-200 border-base-300 rounded-box w-xs self-center border p-4"
    >
      <legend class="fieldset-legend">Run a Traceroute</legend>

      <label class="label" for="maxRounds">Rounds</label>
      <div>
        <input
          id="maxRounds"
          type="number"
          min="1"
          max="200"
          onkeypress={(e) => {
            const isNum = (charValue: string) =>
              charValue.length == 1 && "0123456789".indexOf(charValue) > -1;

            if (!isNum(e.key)) e.preventDefault();
          }}
          bind:value={prefs.maxRounds}
          required
          pattern="^[1-9]\d*$"
          title="Only numbers between 1 and 200"
          class="input input-sm validator"
        />
        <p class="validator-hint text-xs">Must be between 1 to 200</p>
      </div>

      <label class="label" for="ipsearchbox">IP Address or Domain</label>
      <IpSearchBox {search} disabled={formInvalid} />
    </fieldset>
  </div>
{/snippet}
