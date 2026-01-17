<script lang="ts">
  import { APP_VERSION, PLATFORM } from "$lib/bindings";

  import Link from "$lib/components/Link.svelte";
  import AppLogo from "../../../../crates/desktop/icons/icon.svg?raw";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  document.onkeydown = (event) =>
    event.key === "Escape" && getCurrentWindow().close();
</script>

<div class="flex h-screen w-screen flex-col items-center space-y-3 p-4">
  <h1 class="text-2xl">
    Ipmap v{APP_VERSION}
  </h1>

  <div>
    {@html AppLogo}
  </div>

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

          {#if (PLATFORM as string) === "windows"}
            <Link href="https://npcap.com">Npcap</Link>,
          {:else}
            <Link href="https://www.tcpdump.org">Libpcap</Link>,
          {/if}

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
