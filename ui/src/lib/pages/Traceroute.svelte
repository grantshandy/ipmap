<script lang="ts">
  import IpSearchBox from "$lib/components/IpSearchBox.svelte";
  import TraceMap from "$lib/components/TraceMap.svelte";
  import ErrorScreen from "$lib/components/ErrorScreen.svelte";

  import { Channel } from "@tauri-apps/api/core";
  import {
    traceroute,
    isError,
    type Hop,
    type Result,
    type RunTraceroute,
    type Error,
  } from "tauri-plugin-pcap-api";
  import PageSelector from "$lib/components/PageSelector.svelte";

  const MAX_ROUNDS: number = 200;

  let prefs: RunTraceroute = $state({
    rounds: 5,
    ip: "",
  });

  // null => inputting data
  // number => loading round
  // Hop[] => viewing results
  // Error => error message
  let pageState: null | number | Hop[] | Error = $state(null);

  const search = async (input: Result<string, string> | null) => {
    if (input == null || input.status == "error") {
      pageState = input?.error
        ? { kind: "Runtime", message: input.error }
        : null;
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

  let formInvalid = $derived(prefs.rounds < 1 || prefs.rounds > MAX_ROUNDS);
</script>

<div class="flex h-full w-full grow flex-col">
  {#await traceroute.enabled() then enabled}
    {#if enabled.status == "error"}
      <ErrorScreen error={enabled.error} />
    {:else if isError(pageState)}
      <ErrorScreen bind:error={pageState} exitable={true} />
    {:else if typeof pageState == "number"}
      {@render traceLoading(pageState)}
    {:else if Array.isArray(pageState)}
      <TraceMap
        hops={pageState}
        ip={prefs.ip}
        close={() => (pageState = null)}
      />
    {:else}
      {@render tracerouteForm()}
    {/if}
  {/await}
</div>

{#snippet traceLoading(round: number)}
  <div class="flex grow items-center justify-center select-none">
    <div class="space-y-3 text-center">
      <p>Tracing...</p>

      <div class="progress w-56 overflow-hidden rounded-full">
        <div
          class="h-full rounded-r-full bg-white transition-all duration-1000 ease-in-out"
          style={`width: ${Math.min(100, (round / (prefs.rounds - 2)) * 100)}%`}
        ></div>
      </div>
    </div>
  </div>
{/snippet}

{#snippet tracerouteForm()}
  <div class="flex grow items-center justify-center">
    <div class="absolute top-0 left-0 p-2">
      <PageSelector />
    </div>

    <fieldset
      class="fieldset bg-base-200 border-base-300 rounded-box w-xs self-center border p-4"
    >
      <legend class="fieldset-legend">Run a Traceroute</legend>

      <label class="label" for="rounds">Rounds</label>
      <div>
        <input
          id="rounds"
          type="number"
          min="1"
          max={MAX_ROUNDS}
          onkeypress={(e) => {
            const isNum = (charValue: string) =>
              charValue.length == 1 && "0123456789".indexOf(charValue) > -1;

            if (!isNum(e.key)) e.preventDefault();
          }}
          bind:value={prefs.rounds}
          required
          pattern="^[1-9]\d*$"
          title={`Only numbers between 1 and ${MAX_ROUNDS}`}
          class="input input-sm validator"
        />
        <p class="validator-hint text-xs">Must be between 1 to ${MAX_ROUNDS}</p>
      </div>

      <label class="label" for="ipsearchbox">IP Address or Domain</label>
      <IpSearchBox {search} disabled={formInvalid} />
    </fieldset>
  </div>
{/snippet}
