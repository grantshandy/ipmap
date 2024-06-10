import { type Marker, type Map, divIcon, DivIcon, marker, map as mkMap, tileLayer, type LatLngExpression, LayerGroup, layerGroup } from "leaflet";
import "leaflet-providers";
import "leaflet-active-area";

import { lookupIp, type DatabaseInfo, type Location } from "./bindings";
import { writable } from "svelte/store";

export type LocationSelection = {
    loc: Location,
    ips: string[],
    marker: Marker,
}

export enum ApplicationMode {
    CAPTURE,
    SEARCH,
}

type MapStore = {
    selection: LocationSelection | null,
    locations: { [id: string]: LocationSelection },
    connections: Set<string>,
    searchLayer: LayerGroup,
    captureLayer: LayerGroup,
    container: HTMLElement | null,
    instance: Map | null
};

export const mkKey = (loc: Location): string => `${loc.latitude}${loc.longitude}`;
const mkIcon = (num: number, active: boolean): DivIcon => {
    const icon = divIcon({
        html: `<div class="marker-icon ${active ? "bg-info" : "bg-secondary"}"><span>${num}</span></div>`,
        className: "dummyclass",
        iconSize: active ? [30, 30] : [20, 20],
        iconAnchor: active ? [15, 15] : [10, 10],
    });

    return icon;
};

export const map = (() => {
    const { update, set, subscribe } = writable<MapStore>(
        {
            selection: null,
            locations: {},
            connections: new Set(),
            searchLayer: layerGroup(),
            captureLayer: layerGroup(),
            container: null,
            instance: null
        }
    );

    const setContainer = (container: HTMLElement, mode: ApplicationMode) => update((prev) => {
        prev.container = container;
        prev.instance = mkMap(prev.container, { preferCanvas: true });
        prev.instance.setView([30, 0], 2);
        resetView();
        tileLayer.provider("OpenStreetMap.Mapnik").addTo(prev.instance);
        // tileLayer.provider("CartoDB.DarkMatter").addTo(mapInstance);
        // from 'leaflet-active-area'. Fixes a resize bug for map.panTo
        prev.instance.setActiveArea(prev.container);

        setMode(mode);

        return prev;
    });

    const resetView = () => update((prev) => {
        if (prev.instance) {
            // prev.instance.flyTo(
            //     [30, 0],
            //     2,
            // );
            prev.instance.flyTo([30, 0], 2);
        }

        return prev;
    });

    const setMode = (mode: ApplicationMode) => update((prev) => {
        if (!prev.instance) return prev;

        resetView();

        if (mode == ApplicationMode.CAPTURE) {
            prev.searchLayer.removeFrom(prev.instance);
            prev.captureLayer.addTo(prev.instance);
        } else {
            prev.captureLayer.removeFrom(prev.instance);
            prev.searchLayer.eachLayer((layer) => layer.remove());
            prev.searchLayer.addTo(prev.instance);
        }

        setSelection(null);

        return prev;
    });

    const resizeMap = () => update((prev) => {
        if (prev.instance) {
            prev.instance.invalidateSize();
        }

        return prev;
    })

    const setSelection = (location: Location | null, zoom?: number) => update((prev) => {
        if (!prev.instance) return prev;

        if (prev.selection != null) {
            prev.selection.marker
                .setIcon(mkIcon(prev.selection.ips.length, false))
                .setZIndexOffset(50);
        }

        if (location == null) {
            prev.selection = null;
            setTimeout(resizeMap, 10);
            return prev;
        }

        const key: string = mkKey(location);

        if (prev.selection != null && mkKey(prev.selection.loc) == key) {
            prev.selection = null;
            resizeMap();
        } else {
            prev.selection = prev.locations[key];
            prev.selection.marker
                .setIcon(mkIcon(prev.selection.ips.length, true))
                .setZIndexOffset(100);

            const latlng: LatLngExpression = [
                prev.selection.loc.latitude,
                prev.selection.loc.longitude,
            ];

            if (zoom) {
                prev.instance.flyTo(latlng, zoom);
            } else {
                prev.instance.panTo(latlng);
            }
        }

        return prev;
    });

    const addCaptureIp = (ip: string, database: DatabaseInfo) => update((prev) => {
        if (prev.connections.has(ip)) return prev;
        prev.connections.add(ip);

        lookupIp(ip, database).then((location) => {
            if (!location || !prev.instance) return prev;

            const key = mkKey(location);

            if (prev.locations[key]) {
                const loc = prev.locations[key];
                loc.ips.push(ip);
                loc.marker.setIcon(
                    mkIcon(loc.ips.length, loc == prev.selection),
                );
            } else {
                prev.locations[key] = {
                    loc: location,
                    marker: marker([location.latitude, location.longitude], {
                        icon: mkIcon(1, false),
                    })
                        .on("click", (e) => setSelection(location, 5))
                        .addTo(prev.captureLayer),
                    ips: [ip],
                };
            }
        });

        return prev;
    });

    const addSearchIp = (ip: string | null, database: DatabaseInfo) => update((prev) => {
        if (!prev.instance) return prev;

        prev.searchLayer.eachLayer((l) => l.remove());

        if (!ip) {
            resetView();
            setSelection(null);
            return prev;
        };

        (async () => {
            if (!prev.instance) return;

            const loc: Location | null = await lookupIp(ip, database);
            if (!loc) return;

            prev.locations[mkKey(loc)] = {
                loc,
                marker: marker([loc.latitude, loc.longitude], {
                    icon: mkIcon(1, false),
                })
                    .on("click", (e) => setSelection(loc))
                    .addTo(prev.searchLayer),
                ips: [ip],
            };

            prev.selection?.marker.remove();
            prev.selection = null;
            setSelection(loc, 10);
        })();

        return prev;
    });

    return {
        subscribe,
        update,
        set,
        addCaptureIp,
        setSelection,
        resizeMap,
        setContainer,
        addSearchIp,
        setMode,
        resetView
    };
})();