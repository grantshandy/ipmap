<script lang="ts">
  import { pageState } from "$lib/page.svelte";
</script>

<div
  class="bg-base-200 rounded-box flex items-center space-x-2 border border-white px-2 py-1 text-sm"
>
  <input
    id="globe"
    type="checkbox"
    bind:checked={pageState.globe}
    class="toggle toggle-sm"
  />
  <label for="globe"> 3D </label>
</div>
