<script lang="ts">
  import { openInfoWindow } from "../bindings";
  import { theme } from "../utils/theme";

  let open: boolean = false;
</script>

<button
  on:click={() => openInfoWindow($theme)}
  class:btn-active={open}
  class="btn btn-circle btn-primary swap swap-rotate btn-sm h-5 font-mono text-lg"
>
  i
</button>
