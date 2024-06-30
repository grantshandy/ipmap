<script lang="ts">
    import { validateIp, type DatabaseInfo } from "../bindings";
    import { map } from "../map";

    export let loading: string | null;
    export let database: DatabaseInfo | null;

    export let query: string;


    const clearWhitespace = (s: string): string => s.replace(/\s/g, "");

    let queryValid: boolean = false;
    $: validateIp(clearWhitespace(query))
        .then((valid) => {
            queryValid = valid;

            if (!database) return;

            if (valid) {
                map.setSearchIp(clearWhitespace(query), database);
            } else {
                map.setSearchIp(null, database);
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
