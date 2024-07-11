<script lang="ts">
    import { confirm, message, open } from "@tauri-apps/api/dialog";
    import { basename } from "@tauri-apps/api/path";
    import { type DatabaseInfo, geoip } from "../bindings";
    import { database } from "../stores/database";

    export let loading: string | null;

    let databases: DatabaseInfo[] = [];
    geoip.listDatabases().then((db) => {
        databases = db;
        if (databases.length > 0) $database = databases[0];
        loading = null;
    });

    const importDatabase = async () => {
        const dir = await open({
            multiple: false,
            filters: [
                {
                    name: "IPv4 City CSV Database",
                    extensions: ["csv"],
                },
            ],
        });
        if (!dir) return;

        loading = await basename(dir as string);
        const db = await geoip.loadDatabase(dir).catch(() => null);
        loading = null;

        if (!db) return;

        databases = await geoip.listDatabases();
        $database =
            databases.find((l) => l.build_time == db?.build_time) ?? null;
    };

    const dbInfo = () => {
        if (!$database) return;

        const msg =
            `Name: ${$database.name}\n` +
            ($database.attribution_text
                ? `Attribution: ${$database.attribution_text}\n`
                : "") +
            ($database.path ? `Path: ${$database.path}\n` : "") +
            `Build Time: ${$database.build_time}\n` +
            `Locations: ${$database.unique_locations.toLocaleString()}\n` +
            `Unique Strings: ${$database.strings.toLocaleString()}`;

        if ($database.path) {
            // imported database that can be unloaded
            confirm(msg, {
                type: "info",
                title: "Database Info",
                okLabel: "Unload Database",
                cancelLabel: "Close",
            }).then(async (r) => {
                if (!$database?.path) return;

                if (r) {
                    await geoip.unloadDatabase($database.path);
                    databases = await geoip.listDatabases();

                    if (databases.length != 0) {
                        $database = databases[0];
                    } else {
                        $database = null;
                    }
                }
            });
        } else {
            // internal database with no path
            message(msg, {
                type: "info",
                title: "Database Info",
            });
        }
    };
</script>

{#if loading}
    <div
        class="flex items-center"
        class:space-x-3={$database}
        class:space-x-6={!$database}
    >
        <span
            class="italic"
            class:text-sm={$database}
            class:text-lg={!$database}
            class:mx-auto={!$database}>Loading {loading}...</span
        >
        <span
            class="loading loading-spinner"
            class:loading-lg={!$database}
            class:loading-sm={$database}
        ></span>
    </div>
{/if}

{#if $database}
    <button class="btn btn-sm" on:click={dbInfo}>Info</button>
{/if}

{#if databases.length != 0}
    <select
        class="select select-sm select-bordered w-xs"
        bind:value={$database}
    >
        <option selected disabled value={null}>No Database</option>
        {#each databases as database}
            <option value={database}>{database.name}</option>
        {/each}
    </select>
{/if}

{#if !loading}
    <button class="btn btn-sm btn-primary" on:click={importDatabase}
        >Load Database</button
    >
{/if}
