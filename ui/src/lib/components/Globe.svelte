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

<div use:addGlobe class="overflow-none relative min-h-0 grow">
  {@render children?.()}
</div>
