<script lang="ts">
  import { openInfoWindow } from "../bindings";
  import { theme } from "../utils/theme";
  import { event } from "@tauri-apps/api";

  let open: boolean = false;
  $: event.emit("theme-change", $theme);
</script>

<button
  on:click={() => openInfoWindow($theme)}
  class:btn-active={open}
  class="btn btn-circle btn-primary swap swap-rotate btn-sm h-5 font-mono text-lg"
>
  i
</button>
