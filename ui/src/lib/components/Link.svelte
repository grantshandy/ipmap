<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import type { Snippet } from "svelte";

  type Props = {
    children?: Snippet;
    href?: string;
  };

  let { children, href }: Props = $props();

  const onclick = () => {
    if (href) openUrl(href);
  };
</script>

<button class="link" {onclick}>{@render children?.()}</button>
