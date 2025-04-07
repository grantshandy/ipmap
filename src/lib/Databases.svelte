<script lang="ts">
    import { commands, events, type DatabaseInfo } from "../bindings";
    import { open } from "@tauri-apps/plugin-dialog";

    let loading = $state(false);
    let databases: DatabaseInfo[] = $state([]);
    let database: DatabaseInfo | null = $state(null);
    let ipQuery: string = $state("");

    commands.listDatabases().then((dbs) => {
        console.log(dbs);
        databases = dbs;
    });

    events.updateDatabases.listen((ev) => {
        console.log(ev);
        databases = ev.payload;
    });

    const openDatabase = async () => {
        loading = true;

        const file = await open({
            title: "Open IP Geolocation City Database",
            multiple: false,
            directory: false,
            filters: [
                {
                    name: "IP Geolocation City Database",
                    extensions: ["csv", "csv.gz"],
                },
            ],
        });

        if (file) {
            const info = await commands.loadDatabase(file);

            console.log(info);
        }

        loading = false;
    };
</script>

<div class="flex space-x-2">
    <button onclick={openDatabase} class="btn btn-primary">Open</button>

    <select class="select" bind:value={database}>
        <option disabled selected value={null}>Open a Database</option>
        {#each databases as db}
            <option value={db}>{db.path}</option>
        {/each}
    </select>

    {#if loading}
        <span class="loading loading-spinner"></span>
    {/if}
</div>

{#if database}
    <form
        onsubmit={async (ev) => {
            ev.preventDefault();
            if (!database) return;
            console.log(await commands.lookupIp(database, ipQuery));
        }}
    >
        <input type="text" class="input" bind:value={ipQuery} />
        <button class="btn">Search</button>
    </form>
{:else}
    <p>Select a database to make a query</p>
{/if}
