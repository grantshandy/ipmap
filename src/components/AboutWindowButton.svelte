<script lang="ts">
  import { openAboutWindow } from "../bindings";

  let open: boolean = false;
</script>

<button
  on:click={() => openAboutWindow()}
  class:btn-active={open}
  class="btn btn-circle btn-primary swap swap-rotate btn-sm h-5 font-mono text-lg"
>
  i
</button>
