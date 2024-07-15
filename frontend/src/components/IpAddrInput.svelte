<script lang="ts">
  import { geoip } from "../bindings";

  export let error: string | null;
  export let ip: string | null;

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

<input
  class="input input-sm input-bordered w-full grow"
  class:border-error={error}
  placeholder="IP or DNS Address"
  spellcheck="false"
  bind:value={buff}
/>
