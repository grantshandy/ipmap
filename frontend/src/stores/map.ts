import { map as mkMap, type Map, tileLayer, LayerGroup, layerGroup, Marker, marker, DivIcon, divIcon, type LatLngExpression } from "leaflet";
import "leaflet-providers";
import "leaflet-active-area";

import { writable } from "svelte/store";
import { database as databaseStore } from "./database";
import { lookupIp, type DatabaseInfo, type Location } from "../bindings";

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

    return {
        subscribe,
        update,
        set,

        init,
        deinit,

        setSearchIp,
        setSelection,
    };
})();

// creates a MapStore on a div element
const initImpl = (container: HTMLDivElement): MapStore => {
    const arcLayer = layerGroup();
    const markerLayer = layerGroup();

    const inst = mkMap(container, { preferCanvas: false, minZoom: 2, maxZoom: 12, layers: [arcLayer, markerLayer] });
    inst.setView([30, 0], 2);

    tileLayer
        .provider("OpenStreetMap.Mapnik", { noWrap: false })
        .addTo(inst);
    inst.setActiveArea(container); // from "leaflet-active-area", typescript doesn't recognize it.

    return {
        inst,
        ips: new Set(),

        arcLayer,
        markerLayer,

        locations: {},
        selection: null,
    }
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

const mkIcon = (count: number | null, active?: boolean): DivIcon => divIcon({
    html: `<div class="marker-icon ${active ? "bg-info" : "bg-secondary"}"><span>${count ? count : ""}</span></div>`,
    className: "dummyclass",
    iconSize: active ? [25, 25] : [20, 20],
    iconAnchor: active ? [12.5, 12.5] : [10, 10],
});

type LocationKey = string;
const mkLocationKey = (loc: Location) => `${loc.latitude}${loc.longitude}`;

const searchIcon: DivIcon = mkIcon(null, true);
