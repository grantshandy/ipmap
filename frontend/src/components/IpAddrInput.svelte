<script lang="ts">
  import { geoip } from "../bindings";

  export let error: string | null;
  export let onSearch: ((ip: string) => void) | null = null;
  export let disabled: boolean = false;
  let ip: string | null;

  let buff: string;
  $: validate(buff);

  const validate = async (q: string) => {
    if (!q || q.length == 0) {
      error = null;
      ip = null;
      return;
    }

    const validIp = await geoip.validateIp(q);

    if (validIp) ip = q;
    error = validIp ? null : "Invalid Address";

    if (!validIp) {
      const fromHost = await geoip.lookupHost(q);

      if (fromHost) {
        ip = fromHost;
        error = null;
      }
    }
  };
</script>

<div class="w-full join join-horizontal">
  <input
    class="input input-sm input-bordered join-item"
    class:border-error={error}
    placeholder="IP or DNS Address"
    spellcheck="false"
    {disabled}
    bind:value={buff}
    on:submit={() => console.log("submit event")}
  />
  <button
    class="btn btn-primary btn-sm join-item"
    disabled={error != null || !ip || disabled}
    on:click={() => {
      if (onSearch && ip) onSearch(ip);
    }}>Search</button
  >
</div>
