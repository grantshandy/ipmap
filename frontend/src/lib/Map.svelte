<script lang="ts">
    import { fly } from "svelte/transition";
    import { listen } from "@tauri-apps/api/event";
    import { lookupDns, lookupIp } from "../utils";
    import type { LocationSelection } from "../utils";

    import { marker, map, tileLayer, Map, Icon } from "leaflet";
    import "leaflet-providers";
    import "leaflet/dist/leaflet.css";

    import markerIconUrl from "../../node_modules/leaflet/dist/images/marker-icon.png";
    import markerIconRetinaUrl from "../../node_modules/leaflet/dist/images/marker-icon-2x.png";
    import markerShadowUrl from "../../node_modules/leaflet/dist/images/marker-shadow.png";

    Icon.Default.prototype.options.iconUrl = markerIconUrl;
    Icon.Default.prototype.options.iconRetinaUrl = markerIconRetinaUrl;
    Icon.Default.prototype.options.shadowUrl = markerShadowUrl;
    Icon.Default.imagePath = ""; // necessary to avoid Leaflet adds some prefix to image path.

    const countryNames = new Intl.DisplayNames("en", { type: "region" });

    let selection: LocationSelection | null = null;

    let locs: { [id: string]: LocationSelection } = {};
    let mapInstance: Map | null = null;

    let conns: Set<string> = new Set();

    listen("new_connection", (event) => {
        const ip = event.payload as string;

        if (!conns.has(ip)) {
            conns.add(ip);

            lookupIp(ip).then((loc) => {
                if (mapInstance != null && loc != null) {
                    const key = `${loc.latitude}${loc.longitude}`;

                    if (locs[key] != null) {
                        locs[key].ips.push(ip);
                    } else {
                        locs[key] = {
                            loc,
                            marker: marker([loc.latitude, loc.longitude])
                                .on("click", () => (selection = locs[key]))
                                .addTo(mapInstance),
                            ips: [ip],
                        };
                    }
                }
            });
        }
    });

    const mapAction = (container: HTMLDivElement) => {
        mapInstance = map(container, { preferCanvas: true }).setView(
            [30, 0],
            2,
        );
        tileLayer.provider("OpenStreetMap.Mapnik").addTo(mapInstance);

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
</script>

<svelte:window on:resize={resizeMap} />

<div class="w-full h-full flex space-x-2">
    <div class="grow select-none rounded-sm" use:mapAction></div>
    {#if selection}
        <div
            in:fly={{ x: -20, duration: 400 }}
            out:fly={{ x: 20, duration: 400 }}
            class="flex flex-col rounded-sm pl-4 py-4 space-y-2 w-64 overflow-x-auto"
        >
            <div class="w-full flow-root">
                <p class="float-left">Location Information</p>
                <button
                    on:click={() => (selection = null)}
                    class="float-right btn btn-xs rounded-full p-2"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-4 w-4"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        ><path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M6 18L18 6M6 6l12 12"
                        /></svg
                    ></button
                >
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
                            <span class="text-sm">{dns}</span>
                        {/await}
                    </li>
                {/each}
            </ul>
        </div>
    {/if}
</div>
