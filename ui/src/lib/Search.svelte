<script lang="ts">
    import MapView from "$lib/Map.svelte";
    import { isIP, isIPv4, isIPv6 } from "is-ip";

    const validDomainName =
        /^((?!-))(xn--)?[a-z0-9][a-z0-9-_]{0,61}[a-z0-9]{0,1}\.(xn--)?([a-z0-9\-]{1,61}|[a-z0-9-]{1,30}\.[a-z]{2,})$/;

    import { database, type LookupInfo } from "../bindings";
    import { marker, Marker, type Map } from "leaflet";

    const regionNames = new Intl.DisplayNames(["en"], { type: "region" });

    let map: Map | null = $state(null);

    let input = $state("");

    $effect(() => {
        if (input) error = null;
    });

    let ip = $derived(input.replace(/\s/g, ""));

    let validInput = $derived(
        (database.anyEnabled && validDomainName.test(ip)) ||
            (database.ipv4Enabled && isIPv4(ip)) ||
            (database.ipv6Enabled && isIPv6(ip)),
    );

    let loc: LookupInfo | null = $state(null);
    let mrk: Marker | null = $state(null);
    let error: string | null = $state(null);

    $effect(() => {
        if (ip && loc && !validInput) {
            loc = null;
        }
    });

    $effect(() => {
        if (loc) {
            if (!mrk && map) {
                mrk = marker(loc.crd);
            }

            if (map) mrk?.addTo(map);
            mrk?.setLatLng(loc.crd);

            if (map && map.getZoom() > 10) {
                map?.panTo(loc.crd, { duration: 2 });
            } else {
                map?.flyTo(loc.crd, 10, { duration: 2 });
            }
        } else {
            if (mrk) mrk.remove();
        }
    });

    const search = async () => {
        if (!validInput) return;

        const normIp: string | null = isIP(ip)
            ? ip
            : await database.lookupHost(ip);

        if (!normIp) {
            error = "Host IP not found";
            return;
        }

        loc = await database.lookupIp(normIp);

        if (loc == null) {
            error = "IP not found in the database";
        }
    };
</script>

<div class="grow flex flex-col space-y-3">
    <div class="grow flex">
        <MapView bind:map>
            <form
                class="join join-horizontal absolute border top-2 right-2 z-[999] bg-base-300 rounded-box"
                onsubmit={(ev) => {
                    ev.preventDefault();
                    if (ip.length != 0 && validInput) {
                        search();
                    }
                }}
            >
                <input
                    type="text"
                    class="input input-sm join-item"
                    placeholder="IP Address"
                    class:font-mono={ip.length != 0}
                    class:input-error={ip.length != 0 && !validInput}
                    bind:value={input}
                />
                <button
                    class="btn btn-sm btn-primary join-item"
                    disabled={!validInput}
                    type="submit">Search</button
                >
            </form>

            {#if loc}
                <div
                    class="absolute border bottom-2 right-2 bg-base-200 text-sm rounded-box p-2 z-[999] text-right"
                >
                    <p>
                        {`${loc.loc.city ?? "Unknown City"}${loc.loc.region ? `, ${loc.loc.region}` : ""}`}
                    </p>
                    <p class="italic text-xs">
                        {regionNames.of(loc.loc.countryCode)}
                    </p>
                    {#if isIP(ip)}
                        {#await database.lookupDns(ip) then dns}
                            {#if dns}
                                <p class="text-xs">
                                    DNS: <span class="font-mono">{dns}</span>
                                </p>
                            {/if}
                        {/await}
                    {/if}
                </div>
            {/if}

            {#if error}
                <div
                    role="alert"
                    class="alert alert-error py-2 italic text-sm absolute left-2 bottom-2 z-[999]"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-6 w-6 shrink-0 stroke-current"
                        fill="none"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
                        />
                    </svg>
                    <span>Error: {error}</span>
                </div>
            {/if}
        </MapView>
    </div>
</div>
