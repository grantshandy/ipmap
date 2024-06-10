<script lang="ts">
    import { open } from "@tauri-apps/api/dialog";
    import { basename } from "@tauri-apps/api/path";

    import { listDatabases, loadDatabase, type DatabaseInfo } from "../bindings";

    export let database: DatabaseInfo | null;
    export let loading: string | null;

    let databases: DatabaseInfo[] = [];
    listDatabases().then((db) => (databases = db));

    let infoModal: HTMLDialogElement;

    const selectDatabase = async () => {
        const path = await open({
            directory: false,
            multiple: false,
            filters: [{ name: "IPv4-num database", extensions: ["csv"] }],
        });

        if (path == null) {
            return;
        }

        loading = await basename(path.toString());

        const newDatabase = await loadDatabase(path).catch(
            (_) => (loading = null),
        );
        if (newDatabase != null) {
            databases = await listDatabases();
            database =
                databases.find((d) => d.build_time == newDatabase.build_time) ??
                null;
        }

        loading = null;
    };
</script>

<select
    class="select select-bordered select-sm max-w-xs"
    bind:value={database}
    disabled={loading != null || databases.length == 0}
>
    <option disabled selected value={null}>Select Database</option>
    {#each databases as database}
        <option value={database}>{database.name}</option>
    {/each}
</select>
<button
    class="btn btn-sm"
    disabled={loading != null || database == null}
    on:click={() => infoModal.showModal()}
>
    Info
</button>
<button
    class="btn btn-sm btn-primary"
    disabled={loading != null}
    on:click={selectDatabase}
>
    Add
</button>

<dialog bind:this={infoModal} class="modal">
    <div class="modal-box">
        <h3 class="font-bold text-lg">Database Info</h3>
        {#if database}
            <ul class="py-4 list-dic">
                <li>Name: {database.name}</li>
                <li>
                    Distinct Locations: {database.locations.toLocaleString()}
                </li>
                {#if database.path}
                    <li>Path: {database.path}</li>
                {/if}
                {#if database.attribution_text}
                    <li>Attribution: {database.attribution_text}</li>
                {/if}
                <li>Built: {database.build_time}</li>
            </ul>
        {/if}
    </div>
    <form method="dialog" class="modal-backdrop">
        <button>close</button>
    </form>
</dialog>
