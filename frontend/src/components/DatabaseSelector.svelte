<script lang="ts">
    import { open } from "@tauri-apps/api/dialog";
    import {
        listDatabases,
        loadDatabase,
        type DatabaseInfo,
    } from "../bindings";
    import { database } from "../stores/database";

    let databases: DatabaseInfo[] = [];
    listDatabases().then((db) => {
        databases = db;
        if (databases.length > 0) $database = databases[0];
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

        const db = await loadDatabase(dir);
        if (!db) return;

        databases = await listDatabases();
        $database =
            databases.find((l) => l.build_time == db?.build_time) ?? null;
    };
</script>

<select
    class="select select-sm select-bordered"
    bind:value={$database}
    disabled={databases.length == 0}
>
    <option selected disabled value={null}>No Database</option>
    {#each databases as database}
        <option value={database}>{database.name}</option>
    {/each}
</select>

<button class="btn btn-sm btn-primary" on:click={importDatabase}
    >Load Database</button
>
