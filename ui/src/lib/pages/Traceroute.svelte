<script lang="ts">
  import IpSearchBox from "$lib/components/IpSearchBox.svelte";
  import TraceMap from "$lib/components/TraceMap.svelte";
  import ErrorScreen from "$lib/components/ErrorScreen.svelte";

  import { Channel } from "@tauri-apps/api/core";
  import {
    database,
    isTracerouteEnabled,
    runTraceroute,
    type Coordinate,
    type Hop,
    type Result,
    type TracerouteParams,
    type Error,
    isError,
  } from "$lib/bindings";

  const MAX_MAX_ROUNDS: number = 200;

  let prefs: TracerouteParams = $state({
    maxRounds: 5,
    ip: "",
  });

  // null => inputting data
  // number => loading round
  // Hop[] => viewing results
  // Error => error message
  let pageState: null | number | Hop[] | Error = $state(null);

  const search = async (input: Result<string, string> | null) => {
    if (input == null || input.status == "error") {
      pageState = input?.error ? { t: "Ipc", c: input.error } : null;
      return;
    }

    pageState = 0;
    prefs.ip = input.data;
    console.log("running traceroute for ", prefs.ip);
    const res = await runTraceroute(prefs, new Channel((p) => (pageState = p)));
    pageState = res.status == "ok" ? res.data : res.error;
  };

  let myLocation: Coordinate = $state({ lat: 0, lng: 0 });
  database.myLocation().then((l) => {
    if (l) myLocation = l.crd;
  });

  let formInvalid = $derived(
    prefs.maxRounds < 1 || prefs.maxRounds > MAX_MAX_ROUNDS,
  );
</script>

<div class="flex h-full w-full grow flex-col">
  {#await isTracerouteEnabled() then enabled}
    <!-- Various Error Screens -->
    {#if enabled.status == "error"}
      <ErrorScreen error={enabled.error} />
    {:else if enabled.data == false}
      <ErrorScreen error={{ t: "InsufficientPermissions" }} />
    {:else if isError(pageState)}
      <ErrorScreen bind:error={pageState} exitable={true} />

      <!-- Loading Screen -->
    {:else if pageState != null && typeof pageState == "number"}
      {@render traceLoading(pageState)}

      <!-- Traceroute Result -->
    {:else if Array.isArray(pageState)}
      <TraceMap
        hops={pageState}
        {myLocation}
        ip={prefs.ip}
        close={() => (pageState = null)}
      />

      <!-- Input Form -->
    {:else}
      {@render tracerouteForm()}
    {/if}
  {/await}
</div>

{#snippet traceLoading(round: number)}
  <div class="flex grow items-center justify-center select-none">
    <div class="space-y-3 text-center">
      <p>Tracing...</p>
      <progress class="progress w-56" value={round} max={prefs.maxRounds}
      ></progress>
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
