import { map as mkMap, type Map, tileLayer, LayerGroup, layerGroup, Marker, marker, DivIcon, divIcon, type LatLngExpression, DomUtil } from "leaflet";
import "leaflet-providers";
import "leaflet-active-area";

import { writable } from "svelte/store";
import { database as databaseStore } from "./database";
import { lookupIp, myLocation, type ConnectionDirection, type ConnectionInfo, type DatabaseInfo, type Location } from "../bindings";
import { GeodesicLine } from "leaflet.geodesic";

const SELECT_ZOOM = 7;

// a local-read only copy of the database store
let database: DatabaseInfo | null = null;
databaseStore.subscribe((v) => (database = v));

type IpLocation = {
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
};

// a global map store representing the state of the map
export const map = (() => {
    const { subscribe, update, set } = writable<MapStore | null>(null);

    const init = (container: HTMLDivElement) => update(() => initImpl(container));
    const deinit = () => update(deinitImpl);

    const setSearchIp = (ip: string) => update((store) => {
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

    return {
        subscribe,
        update,
        set,

        init,
        deinit,

        setSearchIp,
        setSelection,
        setArcState,
    };
})();

// creates a MapStore on a div element
const initImpl = (container: HTMLDivElement): MapStore => {
    const arcLayer = layerGroup();
    const markerLayer = layerGroup();

    const inst = mkMap(container, { preferCanvas: false, minZoom: 2, maxZoom: 12, layers: [arcLayer, markerLayer] });
    inst.setView([30, 0], 2);

    tileLayer
        .provider("OpenStreetMap.Mapnik", { noWrap: true })
        .addTo(inst);
    inst.setActiveArea(container); // from "leaflet-active-area", typescript doesn't recognize it.

    return {
        inst,
        ips: new Set(),

        arcLayer,
        markerLayer,

        locations: {},
        selection: null,

        currentConnections: {},
    };
};

// cleans up the map by removing it from the div
const deinitImpl = (store: MapStore | null): null => {
    if (store) {
        store.inst.remove();
    }

    return null;
};

const setSearchIpImpl = async (store: MapStore, ip: string) => {
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

const setArcStateImpl = async (store: MapStore, newState: ConnectionInfo[]) => {
    const newStates: { [id: string]: ConnectionInfo } = {};
    for (const i of newState) newStates[i.ip] = i;

    console.log(newStates);

    // remove or change previously added arcs
    for (const prevState of Object.values(store.currentConnections)) {
        const ip = prevState.info.ip;

        if (newStates[ip]) {
            // update direction if needed
            if (prevState.info.direction != newStates[ip].direction) {
                console.log("changing direction");
                prevState.arc.options.className = directionClassNameFromDirection(newStates[ip].direction);
            }
        } else {
            // outdated arcs that no longer exist should be removed
            store.currentConnections[ip].arc.remove();
            delete store.currentConnections[ip];
        }
    }

    const from = await myLocation(database);

    // add arcs that don't already exist
    for (const newState of Object.values(newStates)) {
        // discard already existing connections
        if (store.currentConnections[newState.ip]) continue;

        const to = await lookupIp(newState.ip, database);

        if (!to) continue;

        store.currentConnections[newState.ip] = {
            arc: mkLine(from, to, newState.direction, store.arcLayer),
            info: newState,
        };
    }
};

const mkIcon = (count: number | null, active?: boolean): DivIcon => divIcon({
    html: `<div class="marker-icon ${active ? "bg-info" : "bg-secondary"}"><span>${count ? count : ""}</span></div>`,
    className: "dummyclass",
    iconSize: active ? [25, 25] : [20, 20],
    iconAnchor: active ? [12.5, 12.5] : [10, 10],
});

type LocationKey = string;
const mkLocationKey = (loc: Location) => `${loc.latitude}${loc.longitude}`;

const searchIcon: DivIcon = mkIcon(null, true);

const mkLine = (current: Location, to: Location, direction: ConnectionDirection, map: LayerGroup<any>) => {
    const className = directionClassNameFromDirection(direction);

    const line = new GeodesicLine(
        [
            [current.latitude, current.longitude],
            [to.latitude, to.longitude]
        ],
        {
            weight: 1,
            steps: 3,
            opacity: 0.5,
            className,
        }
    ).addTo(map);

    // please work :)
    DomUtil.addClass(line.getElement() as HTMLElement, className);

    return line;
};

const directionClassNameFromDirection = (direction: ConnectionDirection): string => `line-${direction}`;