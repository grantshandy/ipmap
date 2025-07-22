<script lang="ts">
  import "@openglobus/og/styles";
  import {
    Bing,
    control,
    Globe,
    GlobusRgbTerrain,
    Vector,
  } from "@openglobus/og";
  import type { Snippet } from "svelte";

  type Args = { globe: Globe | null; layers: Vector[]; children?: Snippet };

  let { globe = $bindable(), layers, children }: Args = $props();

  const sat = new Bing(null);

  const addGlobe = (target: HTMLElement) => {
    globe = new Globe({
      target,
      name: "Earth",
      terrain: new GlobusRgbTerrain(),
      atmosphereEnabled: true,
      layers: [sat, ...layers],
      controls: [new control.MouseNavigation()],
      attributionContainer: document.createElement("div"),
    });

    globe.start();

    return {
      destroy: () => globe?.destroy(),
    };
  };
</script>

<div class="overflow-none relative grow">
  <div use:addGlobe class="h-full w-full"></div>
  {@render children?.()}
</div>
