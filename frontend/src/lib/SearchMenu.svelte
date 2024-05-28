<script lang="ts">
    import { validateIp, type DatabaseInfo } from "../utils";
    import { invoke } from "@tauri-apps/api";
    import { map } from "../map";
    import { Address4 } from "ip-address";

    export let loading: string | null;
    export let database: DatabaseInfo | null;

    export let query: string;
    let queryValid: boolean = false;
    $: validateIp(query)
        .then((valid) => {
            queryValid = valid;

            if (!database) return;

            if (valid) {
                map.addSearchIp(query, database);
            } else {
                map.addSearchIp(null, database);
            }
        })
        .catch(() => (queryValid = false));
</script>

<span>Search:</span>
<input
    type="text"
    placeholder="IPv4 Address"
    autocomplete="off"
    disabled={loading != null || !database}
    class="input input-bordered input-sm max-w-sm"
    class:input-error={!queryValid && query && query.length != 0}
    bind:value={query}
/>
