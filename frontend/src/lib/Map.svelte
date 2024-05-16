<script lang="ts">
    import { listen } from "@tauri-apps/api/event";
    import { lookupDns, lookupIp } from "../utils";
    import type { Connection, DatabaseInfo, Location, LocationSelection } from "../utils";

    import { marker, map, tileLayer, Map, DivIcon, divIcon } from "leaflet";
    import "leaflet-providers";
    import "leaflet-active-area";
    import "leaflet/dist/leaflet.css";
    import { fly } from "svelte/transition";
    import CloseIcon from "./CloseIcon.svelte";

    export let database: DatabaseInfo | null;
    export let capturing: string | null;

    const countryNames = new Intl.DisplayNames("en", { type: "region" });
    let mapInstance: Map | null = null;

    const mapAction = (container: HTMLDivElement) => {
        mapInstance = map(container, { preferCanvas: true }).setView(
            [30, 0],
            2,
        );
        tileLayer.provider("OpenStreetMap.Mapnik").addTo(mapInstance);
        // tileLayer.provider("CartoDB.DarkMatter").addTo(mapInstance);

        // from 'leaflet-active-area'. Fixes a resize bug for map.panTo
        mapInstance.setActiveArea(container);

        return {
            destroy: () => {
                if (mapInstance) {
                    mapInstance.remove();
                    mapInstance = null;
                }
            },
        };
    };

    const resizeMap = () => {
        if (mapInstance) {
            mapInstance.invalidateSize();
        }
    };

    const mkKey = (loc: Location): string => `${loc.latitude}${loc.longitude}`;

    const setSelection = (key: string | null) => {
        if (mapInstance == null) {
            return;
        }

        if (selection != null) {
            selection.marker
                .setIcon(mkIcon(selection.ips.length, false))
                .setZIndexOffset(50);
        }

        if (key == null) {
            selection = null;
            setTimeout(resizeMap, 10);
            return;
        }

        if (selection != null && mkKey(selection.loc) == key) {
            selection = null;
            resizeMap();
        } else {
            selection = locs[key];
            selection.marker
                .setIcon(mkIcon(selection.ips.length, true))
                .setZIndexOffset(100);
            // setTimeout(() => {
            // if (mapInstance && selection) {
            mapInstance.panTo([
                selection.loc.latitude,
                selection.loc.longitude,
            ]);
            // }
            // }, 25);
        }
    };

    const mkIcon = (num: number, active: boolean): DivIcon => {
        const icon = divIcon({
            html: `<div class="marker-icon ${active ? "bg-info" : "bg-secondary"}"><span>${num}</span></div>`,
            className: "dummyclass",
            iconSize: active ? [30, 30] : [20, 20],
            iconAnchor: active ? [15, 15] : [10, 10],
        });

        return icon;
    };

    let selection: LocationSelection | null = null;
    let locs: { [id: string]: LocationSelection } = {};
    let conns: Set<string> = new Set();

    listen("new_connection", (event) => {
        const connection = event.payload as Connection;

        if (!conns.has(connection.ip) && capturing == connection.capturing_uuid) {
            conns.add(connection.ip);

            lookupIp(connection.ip, database?.path ?? null).then((loc) => {
                if (mapInstance != null && loc != null) {
                    const key = mkKey(loc);

                    if (locs[key] != null) {
                        const loc = locs[key];

                        loc.ips.push(connection.ip);
                        loc.marker.setIcon(
                            mkIcon(loc.ips.length, loc == selection),
                        );
                    } else {
                        locs[key] = {
                            loc,
                            marker: marker([loc.latitude, loc.longitude], {
                                icon: mkIcon(1, false),
                            })
                                .on("click", (e) => setSelection(key))
                                .addTo(mapInstance),
                            ips: [connection.ip],
                        };
                    }
                }
            });
        }
    });
</script>

<svelte:window on:resize={resizeMap} />

<div class="relative w-full h-full rounded-md overflow-hidden">
    <div
        class="w-full h-full z-20 select-none overflow-hidden"
        use:mapAction
    ></div>
    {#if selection}
        <div
            in:fly={{ duration: 300, x: 20 }}
            out:fly={{ duration: 300, x: 20 }}
            class="absolute right-0 top-0 bottom-0 z-40 flex flex-col w-64 pl-4 pr-2 py-4 space-y-2 bg-base-100/[0.8] overflow-x-auto"
        >
            <div class="flow-root font-bold">
                <h2 class="float-left">Location Information</h2>
                <button
                    class="float-right btn btn-xs btn-circle"
                    on:click={() => setSelection(null)}
                >
                    <CloseIcon />
                </button>
            </div>

            <ul>
                {#if selection.loc.city}
                    <li>City: {selection.loc.city}</li>
                {/if}
                {#if selection.loc.state}
                    <li>State: {selection.loc.state}</li>
                {/if}
                {#if selection.loc.country_code}
                    <li>
                        Country: {countryNames.of(selection.loc.country_code)}
                    </li>
                {/if}
            </ul>

            <p>Addresses</p>
            <ul class="list-disc ml-4">
                {#each selection.ips as ip}
                    <li>
                        {ip}
                        {#await lookupDns(ip) then dns}
                            {#if dns}<span class="text-sm">({dns})</span>{/if}
                        {/await}
                    </li>
                {/each}
            </ul>
        </div>
    {/if}
</div>
