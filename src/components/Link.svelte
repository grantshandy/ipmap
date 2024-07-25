<script lang="ts">
  import { open } from "@tauri-apps/plugin-shell";

  export let href: string;
  export let className: string = "";
</script>

<button class={"link " + className} on:click={() => open(href)}>
  <slot />
</button>
