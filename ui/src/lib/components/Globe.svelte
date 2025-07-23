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

  type Args = {
    globe: Globe | null;
    layers: Vector[];
    children?: Snippet;
    onGlobeInit: (globe: Globe) => void;
  };

  let { globe = $bindable(), layers, children, onGlobeInit }: Args = $props();

  const addGlobe = (target: HTMLElement) => {
    globe = new Globe({
      target,
      name: "Earth",
      terrain: new GlobusRgbTerrain(),
      atmosphereEnabled: true,
      layers: [new Bing(null), ...layers],
      controls: [new control.MouseNavigation()],
      attributionContainer: document.createElement("div"),
    });

    globe.start();

    onGlobeInit(globe);

    return {
      destroy: () => globe?.destroy(),
    };
  };
</script>

<div use:addGlobe class="overflow-none relative min-h-0 grow">
  {@render children?.()}
</div>
