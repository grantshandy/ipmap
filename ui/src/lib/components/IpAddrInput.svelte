<script lang="ts">
  import database from "tauri-plugin-ipgeo-api";
  import { Address4, Address6 } from "ip-address";
  import type { FormEventHandler, HTMLInputAttributes } from "svelte/elements";

  const VALID_DOMAIN_NAME =
    /^((?!-))(xn--)?[a-z0-9][a-z0-9-_]{0,61}[a-z0-9]{0,1}\.(xn--)?([a-z0-9\-]{1,61}|[a-z0-9-]{1,30}\.[a-z]{2,})$/;

  let {
    value = $bindable(),
    class: restClass,
    ...restProps
  }: {
    value?: string | null;
    class?: string;
  } & HTMLInputAttributes = $props();

  let rawValue = $state("");

  const oninput: FormEventHandler<HTMLInputElement> = async () => {
    const currentValue = rawValue;
    const trimmed = currentValue.replace(/\s/g, "");

    if (VALID_DOMAIN_NAME.test(trimmed)) {
      const res = await database.lookupHost(trimmed);

      if (currentValue != rawValue) {
        return;
      }

      value = res.status == "error" ? null : res.data;
      return;
    }

    if (
      (database.ipv4Enabled || database.anyEnabled) &&
      Address4.isValid(trimmed)
    ) {
      value = new Address4(trimmed).correctForm();
      return;
    }

    if (
      (database.ipv6Enabled || database.anyEnabled) &&
      Address6.isValid(trimmed)
    ) {
      value = new Address6(trimmed).correctForm();
      return;
    }

    value = null;
  };
</script>

<input
  type="text"
  class={`input ${restClass || ""}`}
  placeholder="IP Address"
  autocomplete="off"
  class:input-error={rawValue.length > 0 && value == null}
  bind:value={rawValue}
  {oninput}
  {...restProps}
/>
