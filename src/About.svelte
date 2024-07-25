<script lang="ts" type="module">
  import { app, event } from "@tauri-apps/api";
  import { platform } from "@tauri-apps/plugin-os";

  import {
    lightThemes,
    darkThemes,
    defaultDarkTheme,
    defaultLightTheme,
  } from "./themes.json";
  import Link from "./components/Link.svelte";

  export let theme: string;
  let isDark: boolean = darkThemes.includes(theme);

  $: event.emit("set-theme", theme);
</script>

<div
  data-theme={theme}
  class="flex h-screen w-screen select-none flex-col space-y-3 bg-base-100 px-16 py-10 text-center"
>
  <h1 class="pb-4 text-2xl font-bold">
    Ipmap
    {#await app.getVersion() then version}
      v{version}
    {/await}
  </h1>

  <table class="table table-xs border border-base-200">
    <tbody>
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
        <th>Copyright</th>
        <td>
          &copy; 2024
          <Link href="https://github.com/grantshandy">Grant Handy</Link>
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
          <Link href="https://leafletjs.com">LeafletJS</Link>,
          <Link href="https://osm.org">OpenStreetMap Contributors</Link>,
          <Link href="https://tauri.app">Tauri</Link>,
          <Link href="https://svelte.dev">Svelte</Link>,

          {#if platform() == "windows"}
            <Link href="https://npcap.com">Npcap</Link>,
          {:else}
            <Link href="https://www.tcpdump.org">Libpcap</Link>,
          {/if}

          <Link href="https://daisyui.com">DaisyUI</Link>
        </td>
      </tr>
    </tbody>
  </table>

  <table class="table table-xs border border-base-200">
    <tbody>
      <tr>
        <th>Theme</th>
        <td class="flex">
          <select
            class="select select-xs"
            bind:value={isDark}
            on:change={() =>
              (theme = isDark ? defaultDarkTheme : defaultLightTheme)}
          >
            <option value={true}>Dark</option>
            <option value={false}>Light</option>
          </select>
          <select class="select select-xs grow" bind:value={theme}>
            {#each isDark ? darkThemes : lightThemes as theme}
              <option value={theme}>{theme}</option>
            {/each}
          </select>
        </td>
      </tr>
    </tbody>
  </table>

  <div class="flex grow flex-col-reverse">
    <Link href="https://buymeacoffee.com/granthandy" className="italic"
      >Donate to help support the project!</Link
    >
  </div>
</div>
