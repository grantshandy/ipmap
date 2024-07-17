<script lang="ts">
  import type { LocationInfo } from "../bindings";

  const countryNames = new Intl.DisplayNames("en", { type: "region" });

  export let info: LocationInfo;
</script>

{#if info.city}
  {info.city},
{/if}
{#if info.state}
  {info.state},
{/if}
{countryNames.of(info.country_code)}
