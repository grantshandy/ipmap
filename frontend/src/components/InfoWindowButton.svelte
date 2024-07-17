<script lang="ts">
  import { WebviewWindow } from "@tauri-apps/api/window";
  import { theme } from "../utils/theme";
  import { emit } from "@tauri-apps/api/event";

  const info = WebviewWindow.getByLabel("info");
  let open: boolean = false;

  if (info)
    info.onCloseRequested((_) => {
      open = false;
    });

  $: if (info && $theme) emit("theme", $theme);
</script>

<button
  on:click={() => {
    if (info) {
      open = true;
      info.show();
    }
  }}
  class:btn-active={open}
  class="btn btn-circle btn-primary swap swap-rotate btn-sm h-5 font-mono text-lg"
>
  i
</button>
