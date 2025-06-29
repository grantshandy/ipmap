<script lang="ts">
  import Capture from "./Capture.svelte";

  import { newPcapInstance } from "../../bindings";
</script>

{#await newPcapInstance() then pcap}
  {#if typeof pcap == "string"}
    <div class="flex grow items-center justify-center">
      <div class="rounded-box bg-error max-w-96 space-y-2 px-3 py-2">
        <h1 class="text-lg font-semibold">
          Error Loading <code>libpcap</code>:
        </h1>
        <p class="text-sm">
          <code>{pcap}</code>
        </p>
      </div>
    </div>
  {:else if pcap != null}
    <Capture {pcap} />
  {/if}
{/await}
