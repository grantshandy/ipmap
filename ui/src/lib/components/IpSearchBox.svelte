<script lang="ts">
  import { database, type Result } from "$lib/bindings";
  import { Address4, Address6 } from "ip-address";

  const validDomainName =
    /^((?!-))(xn--)?[a-z0-9][a-z0-9-_]{0,61}[a-z0-9]{0,1}\.(xn--)?([a-z0-9\-]{1,61}|[a-z0-9-]{1,30}\.[a-z]{2,})$/;

  type SearchCallback = (ip: Result<string, string> | null) => void;

  let { search, disabled }: { search: SearchCallback; disabled?: boolean } =
    $props();

  let input = $state("");
  let trimmedInput = $derived(input.replace(/\s/g, ""));

  let isDomainName: boolean = $derived(validDomainName.test(trimmedInput));

  let ipv4: string | null = $derived(
    database.ipv4Enabled && Address4.isValid(trimmedInput)
      ? new Address4(trimmedInput).correctForm()
      : null,
  );
  let ipv6: string | null = $derived(
    database.ipv6Enabled && Address6.isValid(trimmedInput)
      ? new Address6(trimmedInput).correctForm()
      : null,
  );

  let validInput: boolean = $derived(
    isDomainName || ipv4 != null || ipv6 != null,
  );

  $effect(() => {
    if (!validInput) search(null);
  });

  const searchWrapper = async () => {
    if (!validInput || disabled) return;

    let ip: string | null = null;

    if (isDomainName) {
      const lookup = await database.lookupHost(trimmedInput);

      if (lookup.status == "error" || lookup.data == null) {
        search({
          status: "error",
          error: `Domain name "${trimmedInput}" not found`,
        });
        return;
      }

      ip = lookup.data;
    } else {
      ip = ipv4 ?? ipv6;
    }

    // shouldn't happen, ipv4, ipv6, and isDomainName are mutually exclusive
    if (!ip) return;

    input = trimmedInput;
    search({ status: "ok", data: ip });
  };
</script>

<form class="join join-horizontal select-none" onsubmit={searchWrapper}>
  <input
    id="ipsearchbox"
    type="text"
    class="input input-sm join-item"
    placeholder="IP Address"
    {disabled}
    class:input-error={trimmedInput.length != 0 && !validInput}
    bind:value={input}
  />
  <button
    class="btn btn-sm btn-primary join-item"
    disabled={!validInput || disabled}
    type="submit">Search</button
  >
</form>
