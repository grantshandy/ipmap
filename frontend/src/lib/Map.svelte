<script lang="ts">
    import { Pane, Splitpanes } from "svelte-splitpanes";
    import { listen } from "@tauri-apps/api/event";
    import { lookupDns, lookupIp } from "../utils";
    import type { Location, LocationSelection } from "../utils";

    import {
        marker,
        map,
        tileLayer,
        Map,
        DivIcon,
        divIcon,
        type LeafletMouseEvent,
    } from "leaflet";
    import "leaflet-providers";
    import "leaflet-active-area";
    import "leaflet/dist/leaflet.css";

    const countryNames = new Intl.DisplayNames("en", { type: "region" });
    let mapInstance: Map | null = null;

    const mapAction = (container: HTMLDivElement) => {
        mapInstance = map(container, { preferCanvas: true }).setView(
            [30, 0],
            3,
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
            console.log("resize");
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
            return;
        }

        if (
            selection != null &&
            mkKey(selection.loc) == key
        ) {
            selection = null;
        } else {
            selection = locs[key];
            selection.marker
                .setIcon(mkIcon(selection.ips.length, true))
                .setZIndexOffset(100);
            setTimeout(() => {
                if (mapInstance && selection) {
                    mapInstance.panTo([
                        selection.loc.latitude,
                        selection.loc.longitude,
                    ]);
                }
            }, 25);
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
        const ip = event.payload as string;

        if (!conns.has(ip)) {
            conns.add(ip);

            lookupIp(ip).then((loc) => {
                if (mapInstance != null && loc != null) {
                    const key = mkKey(loc);

                    if (locs[key] != null) {
                        const loc = locs[key];

                        loc.ips.push(ip);
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
                            ips: [ip],
                        };
                    }
                }
            });
        }
    });
</script>

<svelte:window on:resize={resizeMap} />

<Splitpanes
    horizontal={false}
    theme="dummy"
    on:resize={resizeMap}
    on:pane-remove={resizeMap}
    on:splitter-click={() => {
        setSelection(null);
        resizeMap();
    }}
    class="w-full h-full"
>
    <Pane size={100}>
        <div class="w-full h-full select-none rounded-sm" use:mapAction></div>
    </Pane>
    {#if selection}
        <Pane
            class="flex flex-col rounded-sm pl-4 py-4 space-y-2 w-full h-full overflow-x-auto"
            size={30}
            snapSize={10}
            maxSize={40}
        >
            <p>Location Information</p>

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
        </Pane>
    {/if}
</Splitpanes>
