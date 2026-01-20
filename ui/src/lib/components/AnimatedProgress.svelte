<script lang="ts">
  let {
    value = $bindable(),
    class: restClass,
  }: {
    value?: number | null;
    class?: string;
  } = $props();
</script>

<div class={`progress overflow-hidden rounded-full ${restClass || ""}`}>
  {#if value}
    <div
      class="h-full rounded-r-full bg-white transition-all duration-1000 ease-in-out"
      style={`width: ${Math.min(100, value * 100)}%`}
    ></div>
  {/if}
</div>
