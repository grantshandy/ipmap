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

<form
  class="join join-horizontal w-full"
  on:submit|preventDefault={() => {
    if (ip && onSearch) onSearch(ip);
  }}
>
  <input
    class="input input-sm join-item input-bordered w-full"
    class:border-error={error}
    placeholder="IP or DNS Address"
    spellcheck="false"
    {disabled}
    bind:value={buff}
    on:submit={(v) => console.log(v)}
  />
  <button
    type="submit"
    class="btn btn-primary join-item input-bordered btn-sm"
    disabled={error != null || !ip || disabled}>Search</button
  >
</form>
