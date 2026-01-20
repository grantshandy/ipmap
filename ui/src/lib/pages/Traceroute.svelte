<script lang="ts">
  import IpAddrInput from "$lib/components/IpAddrInput.svelte";
  import TraceMap from "$lib/components/TraceMap.svelte";
  import ErrorScreen from "$lib/components/ErrorScreen.svelte";
  import PageSelector from "$lib/components/PageSelector.svelte";

  import {
    traceroute,
    isError,
    type Hop,
    type Error,
  } from "tauri-plugin-pcap-api";

  import { Channel } from "@tauri-apps/api/core";
  import AnimatedProgress from "$lib/components/AnimatedProgress.svelte";

  const MAX_ROUNDS: number = 200;

  let rounds: number = $state(5);
  let ip: string | null = $state(null);
  let searchLoading: boolean = $state(false);

  let disabled = $derived(ip === null || rounds < 1 || rounds > MAX_ROUNDS);

  // null => inputting data
  // number => loading round
  // Hop[] => viewing results
  // Error => error message
  let pageState: null | number | Hop[] | Error = $state(null);

  const search = async () => {
    if (!ip) return;

    // start loading screen
    pageState = 0;

    const res = await traceroute.run(
      { ip, rounds },
      new Channel((p) => (pageState = p)),
    );
    pageState = res.status == "ok" ? res.data : res.error;
  };

  const reset = () => {
    ip = null;
    rounds = 5;
    pageState = null;
  };

  reset();
</script>

<div class="flex h-full w-full grow flex-col">
  {#await traceroute.enabled() then enabled}
    {#if enabled.status == "error"}
      <ErrorScreen error={enabled.error} />
    {:else if isError(pageState)}
      <ErrorScreen bind:error={pageState} exitable={true} />
    {:else if typeof pageState == "number"}
      {@render traceLoading(pageState)}
    {:else if Array.isArray(pageState) && ip != null}
      <TraceMap hops={pageState} close={reset} {ip} />
    {:else}
      {@render tracerouteForm()}
    {/if}
  {/await}
</div>

{#snippet traceLoading(round: number)}
  <div class="flex grow items-center justify-center select-none">
    <div class="space-y-3 text-center">
      <p>Tracing...</p>
      <AnimatedProgress value={round / (round - 2)} class="w-56" />
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
          bind:value={rounds}
          required
          pattern="^[1-9]\d*$"
          title={`Only numbers between 1 and ${MAX_ROUNDS}`}
          class="input input-sm validator"
        />
        <p class="validator-hint text-xs">Must be between 1 to ${MAX_ROUNDS}</p>
      </div>

      <label class="label" for="ipsearchbox">
        <span class="grow">IP Address or Domain Name</span>
        {#if searchLoading}
          <span class="loading loading-spinner loading-xs"></span>
        {/if}
      </label>
      <IpAddrInput
        id="ipsearchbox"
        class="input-sm"
        placeholder="wikipedia.org"
        bind:value={ip}
        bind:loading={searchLoading}
        onchange={search}
      />

      <button class="btn btn-primary btn-sm mt-4" onclick={search} {disabled}>
        Search
      </button>
    </fieldset>
  </div>
{/snippet}
