<script lang="ts">
  import {
    renderDeviceName,
    type Pcap,
    type SessionCallbacks,
  } from "$lib/bindings";

  let {
    pcap,
    callbacks,
    globe = $bindable(),
  }: {
    pcap: Pcap;
    callbacks: SessionCallbacks;
    globe?: boolean;
  } = $props();
</script>

<div class="flex items-center space-x-2">
  <input id="globe" type="checkbox" bind:checked={globe} class="toggle" />
  <label for="globe">3D</label>

  <div class="join join-horizontal rounded-box border">
    <select
      class="join-item select select-sm w-48"
      disabled={pcap.status.capture != null}
      bind:value={pcap.device}
    >
      {#each pcap.status.devices as device}
        <option value={device} disabled={!device.ready} selected>
          {#await renderDeviceName(device) then name}
            {name}
          {/await}
        </option>
      {/each}
    </select>
    {#if pcap.status.capture}
      <button
        onclick={() => pcap.stopCapture()}
        class="join-item btn btn-sm btn-error"
      >
        Stop Capture
      </button>
    {:else}
      <button
        onclick={() => pcap.startCapture(callbacks)}
        class="join-item btn btn-sm btn-primary"
        disabled={pcap.device == null}
      >
        Start Capture
      </button>
    {/if}
  </div>
</div>
