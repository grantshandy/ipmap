<script lang="ts">
  import { utils } from "$lib/bindings";
  import Link from "$lib/components/Link.svelte";
</script>

<div class="flex h-screen w-screen flex-col items-center space-y-3 p-4">
  <h1 class="text-2xl">
    Ipmap
    {#await utils.version() then version}
      v{version}
    {/await}
  </h1>

  <table class="table-xs border-base-200 table border">
    <tbody>
      <tr>
        <th>Copyright</th>
        <td>
          &copy; 2025
          <Link href="https://github.com/grantshandy">Grant Handy</Link>
        </td>
      </tr>
      <tr>
        <th>Source Code</th>
        <td>
          <Link href="https://github.com/grantshandy/ipmap">
            Github Repository
          </Link>
        </td>
      </tr>
      <tr>
        <th>Issue Tracker</th>
        <td>
          <Link href="https://github.com/grantshandy/ipmap/issues">
            Submit an Issue
          </Link>
        </td>
      </tr>
      <tr>
        <th>License</th>
        <td>
          <Link href="https://github.com/grantshandy/ipmap/blob/main/LICENSE">
            GNU General Public License v3.0
          </Link>
        </td>
      </tr>
      <tr>
        <th>Special Thanks</th>
        <td>
          <Link href="https://trippy.rs">Trippy</Link>,
          <Link href="https://leafletjs.com">LeafletJS</Link>,
          <Link href="https://osm.org">OpenStreetMap Contributors</Link>,
          <Link href="https://tauri.app">Tauri</Link>,
          <Link href="https://svelte.dev">Svelte</Link>,

          {#await utils.platform() then platform}
            {#if platform == "windows"}
              <Link href="https://npcap.com">Npcap</Link>,
            {:else}
              <Link href="https://www.tcpdump.org">Libpcap</Link>,
            {/if}
          {/await}

          <Link href="https://daisyui.com">DaisyUI</Link>
        </td>
      </tr>
    </tbody>
  </table>

  <div class="grow"></div>

  <p class="italic">
    <Link href="https://buymeacoffee.com/granthandy">
      Donate to help support the project!</Link
    >
  </p>
</div>
