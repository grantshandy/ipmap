<script lang="ts">
  import { type DatabaseInfo, type IpType } from "../bindings";
  import { database } from "../utils/database";
  import { confirm, message } from "@tauri-apps/plugin-dialog";

  const dbInfo = (db: IpType) => {
    let info: DatabaseInfo;

    if (db == "IPv4") {
      if (!$database.ipv4) return;
      info = $database.ipv4;
    } else {
      if (!$database.ipv6) return;
      info = $database.ipv6;
    }

    const msg =
      `Name: ${info.name}\n` +
      (info.attribution_text ? `Attribution: ${info.attribution_text}\n` : "") +
      `Type: ${info.kind}\n` +
      (info.query != "Internal" ? `Path: ${info.query.Loaded}\n` : "") +
      `Build Time: ${info.build_time.replace("+00:00:00", "").split(".", 2)[0]}\n` +
      `Locations: ${info.unique_locations.toLocaleString()}\n` +
      `Unique Strings: ${info.strings.toLocaleString()}`;

    if (info.query != "Internal") {
      // imported database that can be unloaded
      confirm(msg, {
        kind: "info",
        title: "Database Info",
        okLabel: "Unload Database",
        cancelLabel: "Close",
      }).then((r) => {
        if (r) database.unloadDatabase(info);
      });
    } else {
      // internal database with no path
      message(msg, {
        kind: "info",
        title: "Database Info",
      });
    }
  };
</script>

{#if $database.loading}
  <div
    class="absolute bottom-8 left-2 z-[999] flex items-center space-x-3 rounded-r-box border-y border-r border-base-300 bg-base-100/[0.8] px-4 py-2"
  >
    <span class="text-sm italic">Loading {$database.loading}...</span>
    <span class="loading loading-spinner loading-sm"></span>
  </div>
{/if}

{#if $database.ipv4dbs.length != 0}
  <div class="join join-horizontal">
    <button
      class="btn join-item select-bordered btn-sm"
      disabled={$database.ipv4 == null}
      on:click={() => dbInfo("IPv4")}>Info</button
    >
    <select
      class="w-xs join-item select select-bordered select-sm"
      bind:value={$database.ipv4}
    >
      <option selected disabled value={null}>No IPv4 Database</option>
      {#each $database.ipv4dbs as database}
        <option value={database}>{database.name}</option>
      {/each}
    </select>
  </div>
{/if}

{#if $database.ipv6dbs.length != 0}
  <div class="join join-horizontal">
    <button
      class="btn join-item select-bordered btn-sm"
      disabled={$database.ipv6 == null}
      on:click={() => dbInfo("IPv6")}>Info</button
    >
    <select
      class="w-xs join-item select select-bordered select-sm"
      bind:value={$database.ipv6}
    >
      <option selected disabled value={null}>No IPv6 Database</option>
      {#each $database.ipv6dbs as database}
        <option value={database}>{database.name}</option>
      {/each}
    </select>
  </div>
{/if}

<button
  class="btn btn-square btn-primary btn-sm"
  disabled={$database.loading != null}
  on:click={database.importDatabase}
>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 24 24"
    class="h-5 w-5 fill-current stroke-current stroke-0"
    ><path d="M9,16V10H5L12,3L19,10H15V16H9M5,20V18H19V20H5Z" /></svg
  >
</button>
