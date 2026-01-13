<script lang="ts">
  import { pageState } from "$lib/page.svelte";
</script>

<select class="select select-sm join-item max-w-40" bind:value={pageState.page}>
  <option value="search">Location Search</option>
  <option value="capture">Monitor Network</option>
  <option value="trace">Traceroute</option>
</select>
