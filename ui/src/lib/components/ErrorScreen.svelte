<script lang="ts">
  import Link from "./Link.svelte";

  import { printError as printError, utils, type Error } from "$lib/bindings";

  let {
    error = $bindable(),
    exitable,
  }: { error: Error | null; exitable?: boolean } = $props();
</script>

{#if error}
  <div class="flex grow items-center justify-center">
    <div class="rounded-box bg-error max-w-110 space-y-2 px-3 py-2">
      {#if error.kind === "InsufficientPermissions"}
        {@render insufficientPermissionsInfo(error.message)}
      {:else if error.kind === "LibLoading"}
        {@render libLoadingErrorInfo(error.message)}
      {:else}
        <h1 class="text-lg font-semibold">Error</h1>
        {#await printError(error) then error}
          <pre class="overflow-x-auto text-sm">{error}</pre>
        {/await}
        {@render exitButton()}
      {/if}
    </div>
  </div>
{/if}

{#snippet insufficientPermissionsInfo(path: string | null)}
  <h1 class="text-lg font-semibold">
    Child Process Has Insufficient Network Permissions
  </h1>
  {#await utils.platform() then platform}
    {#if platform === "linux"}
      <p class="text-sm">
        In order to perform this action, you must enable network capabilities on
        the child executable.
      </p>
      <pre
        class="bg-base-100 bg-opacity-80 overflow-x-auto rounded-sm px-2 py-3 text-xs"># setcap cap_net_raw,cap_net_admin=eip {path}</pre>
      <button class="btn btn-sm" onclick={() => window.location.reload()}
        >Retry</button
      >
    {:else}
      <p class="text-sm">Try restarting the program as an administrator.</p>
    {/if}
  {/await}
{/snippet}

{#snippet libLoadingErrorInfo(message: string | null)}
  {#await utils.platform() then platform}
    <h1 class="text-lg font-semibold">
      Failed to Load {platform === "windows" ? "Npcap" : "libpcap"} Driver
    </h1>
    <p class="text-sm">
      {#if platform === "windows"}
        You should be able to fix this by installing
        <Link href="https://npcap.com/">Npcap</Link>
        from their website and restarting your computer.
      {:else if platform === "linux"}
        Install
        <Link href="https://repology.org/project/libpcap/versions">libpcap</Link
        >
        using your Linux distribution's package manager and make sure it is visible
        to this program.
      {:else}
        libpcap should be installed on MacOS by default, please contact the
        developer.
      {/if}
    </p>
    <p class="text-xs">Message: <code>{message ?? "No Message"}</code></p>
  {/await}
{/snippet}

{#snippet exitButton()}
  {#if exitable}
    <div class="flex flex-row-reverse">
      <button class="btn btn-sm" onclick={() => (error = null)}>Ok</button>
    </div>
  {/if}
{/snippet}
