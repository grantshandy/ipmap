<script lang="ts" type="module">
  import { app, event } from "@tauri-apps/api";
  import { open } from "@tauri-apps/plugin-shell";
  import {
    lightThemes,
    darkThemes,
    defaultDarkTheme,
    defaultLightTheme,
  } from "./themes.json";

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
          <button
            on:click={() => open("https://github.com/grantshandy/ipmap")}
            class="link"
          >
            Github Repository
          </button>
        </td>
      </tr>
      <tr>
        <th>Issue Tracker</th>
        <td>
          <button
            on:click={() => open("https://github.com/grantshandy/ipmap/issues")}
            class="link"
          >
            Submit an Issue
          </button>
        </td>
      </tr>
      <tr>
        <th>Copyright</th>
        <td>
          &copy; 2024
          <button
            on:click={() => open("https://github.com/grantshandy/")}
            class="link"
          >
            Grant Handy
          </button>
        </td>
      </tr>
      <tr>
        <th>License</th>
        <td>
          <button
            on:click={() =>
              open("https://github.com/grantshandy/ipmap/blob/main/LICENSE")}
            class="link"
          >
            GNU General Public License v3.0
          </button>
        </td>
      </tr>
      <tr>
        <th>Special Thanks</th>
        <td>
          <button on:click={() => open("https://leafletjs.com")} class="link">
            LeafletJS
          </button>,
          <button on:click={() => open("https://tauri.app")} class="link">
            Tauri
          </button>,
          <button on:click={() => open("https://svelte.dev")} class="link">
            Svelte
          </button>,
          <button on:click={() => open("https://www.tcpdump.org")} class="link">
            Libpcap
          </button>,
          <button on:click={() => open("https://npcap.com")} class="link">
            Npcap
          </button>,
          <button on:click={() => open("https://daisyui.com")} class="link">
            DaisyUI
          </button>
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
    <button
      on:click={() => open("https://buymeacoffee.com/granthandy")}
      class="link italic"
    >
      Donate to help support the project!
    </button>
  </div>
</div>
