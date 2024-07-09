import { map as mkMap, type Map, tileLayer, LayerGroup, layerGroup, Marker, marker, DivIcon, divIcon, type LatLngExpression, type TileLayerOptions } from "leaflet";
import "leaflet-edgebuffer";
import "leaflet-active-area";

import { writable } from "svelte/store";
import { database as databaseStore } from "./database";
import { lookupIp, myLocation, type ConnectionDirection, type ConnectionInfo, type DatabaseInfo, type Location } from "../bindings";
import { GeodesicLine } from "leaflet.geodesic";

const SELECT_ZOOM = 7;

// a local-read only copy of the database store
let database: DatabaseInfo | null = null;
databaseStore.subscribe((v) => (database = v));

export type IpLocation = {
    marker: Marker,
    info: Location,
    ips: Set<string>,
};

type MapStore = {
    inst: Map,
    ips: Set<string>,

    arcLayer: LayerGroup,
    markerLayer: LayerGroup,

    locations: { [id: LocationKey]: IpLocation },
    selection: IpLocation | null,

    // id: ip
    currentConnections: { [id: string]: { info: ConnectionInfo, arc: GeodesicLine } },

    locationMarker: Marker | null,
};

// a global map store representing the state of the map
export const map = (() => {
    const { subscribe, update, set } = writable<MapStore | null>(null);

    const init = (container: HTMLDivElement) => update(() => initImpl(container));
    const deinit = () => update(deinitImpl);

    const setSearchIp = (ip: string | null) => update((store) => {
        if (store) setSearchIpImpl(store, ip);
        return store;
    });

    const setSelection = (selection: IpLocation) => update((store) => {
        if (store) setSelectionImpl(store, selection);
        return store;
    });

    const setArcState = (state: ConnectionInfo[]) => update((store) => {
        if (store) setArcStateImpl(store, state);
        return store;
    });

    const addIp = (ip: string) => update((store) => {
        if (store) addIpImpl(store, ip);
        return store;
    });

    const invalidateSize = () => update((store) => {
        if (!store) return null;

        store.inst.invalidateSize();

        return store;
    });

    const resetView = () => update((store) => {
        if (store) resetMapView(store.inst);
        return store;
    });

    return {
        subscribe,
        update,
        set,

        init,
        deinit,

        setSearchIp,
        setSelection,
        setArcState,
        addIp,
        invalidateSize,
        resetView
    };
})();

// creates a MapStore on a div element
const initImpl = (container: HTMLDivElement): MapStore => {
    const arcLayer = layerGroup();
    const markerLayer = layerGroup();

    const inst = mkMap(container, { preferCanvas: false, minZoom: 2, maxZoom: 12, layers: [arcLayer, markerLayer] });
    resetMapView(inst);

    tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
        attribution: '&copy; OSM Contributors',
        noWrap: true,
        edgeBufferTiles: 5,
    } as any).addTo(inst);

    return {
        inst,
        ips: new Set(),

        arcLayer,
        markerLayer,

        locations: {},
        selection: null,

        currentConnections: {},
        locationMarker: null,
    };
};

// cleans up the map by removing it from the div
const deinitImpl = (store: MapStore | null): null => {
    if (store) {
        store.inst.remove();
    }

    return null;
};

const setSearchIpImpl = async (store: MapStore, ip: string | null) => {
    if (!ip) {
        if (!store.selection) return;

        store.selection?.marker.remove();
        store.selection = null;

        return;
    }

    const location = await lookupIp(ip, database);

    if (!location) {
        console.warn("couldn't find location for " + ip);
        return;
    }

    const key = mkLocationKey(location);

    resetMarkersImpl(store);
    store.locations[key] = {
        marker: marker(
            [location.latitude, location.longitude],
            { icon: searchIcon }
        ).addTo(store.markerLayer),
        info: location,
        ips: new Set([ip]),
    };
    map.setSelection(store.locations[key]);
};

const resetMarkersImpl = (store: MapStore) => {
    store.markerLayer.eachLayer((l) => l.remove());
    store.locations = {};
};

const setSelectionImpl = (store: MapStore, selection: IpLocation) => {
    const latlng: LatLngExpression = [
        selection.info.latitude,
        selection.info.longitude,
    ];

    if (store.inst.getZoom() < SELECT_ZOOM) {
        store.inst.flyTo(latlng, SELECT_ZOOM);
    } else {
        store.inst.panTo(latlng);
    }
    store.selection = selection;
};

const setArcStateImpl = (store: MapStore, newState: ConnectionInfo[]) => {
    addLocationMarkerIfNotExists(store);

    const newStates: { [id: string]: ConnectionInfo } = {};
    for (const i of newState) newStates[i.ip] = i;

    // remove or change previously added arcs
    for (const prevState of Object.values(store.currentConnections)) {
        const ip = prevState.info.ip;

        if (newStates[ip]) {
            // update direction if needed
            if (prevState.info.direction != newStates[ip].direction) {
                prevState.arc.options.className = newStates[ip].direction as string;
            }
        } else {
            // outdated arcs that no longer exist should be removed
            store.currentConnections[ip].arc.remove();
            delete store.currentConnections[ip];
        }
    }

    myLocation(database).then((from) => {
        // add arcs that don't already exist
        for (const newState of Object.values(newStates)) {
            // discard already existing connections
            if (store.currentConnections[newState.ip]) continue;

            lookupIp(newState.ip, database).then((to) => {
                if (!to) return;

                store.currentConnections[newState.ip] = {
                    arc: mkLine(from, to, newState.direction, store.arcLayer),
                    info: newState,
                };
            });
        }
    });
};

const addIpImpl = async (store: MapStore, ip: string) => {
    if (store.ips.has(ip)) return;
    store.ips.add(ip);

    const location = await lookupIp(ip, database);

    if (!location) return;

    const key = mkLocationKey(location);

    const iploc = store.locations[key];

    if (iploc) {
        iploc.ips.add(ip);
        iploc.marker.setIcon(mkIcon(iploc.ips.size, false));
    } else {
        store.locations[key] = {
            info: location,
            ips: new Set([ip]),
            marker: marker(
                [location.latitude, location.longitude],
                { icon: mkIcon(1, false) }
            ).addTo(store.markerLayer)
        };
    }
};

const addLocationMarkerIfNotExists = (store: MapStore) => {
    if (!store.locationMarker) {
        store.locationMarker = marker([0, 0], {
            icon: mkIcon(null, true)
        });

        myLocation(database).then((location) => {
            if (!store.locationMarker) return;

            store
                .locationMarker
                .setLatLng([location.latitude, location.longitude])
                .addTo(store.markerLayer);
        });
    }
};

const resetMapView = (map: Map) => {
    map.setView([25, 0], 2);
};

export const mkIcon = (count: number | null, active?: boolean): DivIcon => divIcon({
    html: `<div class="marker-icon ${active ? "bg-primary" : "bg-secondary"}">${count ? count : ""}</div>`,
    className: "dummyclass",
    iconSize: [20, 20],
    iconAnchor: [10, 10],
});

type LocationKey = string;
const mkLocationKey = (loc: Location) => `${loc.latitude}${loc.longitude}`;

const searchIcon: DivIcon = mkIcon(null, true);

const mkLine = (current: Location, to: Location, direction: ConnectionDirection, map: LayerGroup<any>) => {
    const line = new GeodesicLine(
        [
            [current.latitude, current.longitude],
            [to.latitude, to.longitude]
        ],
        {
            weight: 1,
            steps: 3,
            opacity: 0.5,
            className: direction,
        }
    ).addTo(map);

    return line;
};
