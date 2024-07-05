import { map as mkMap, type Map, tileLayer, LayerGroup, layerGroup } from "leaflet";
import "leaflet-providers";
import "leaflet-active-area";

import { writable } from "svelte/store";
import { database as databaseStore } from "./database";
import { lookupIp, type DatabaseInfo } from "../bindings";

// a local-read only copy of the database store
let database: DatabaseInfo | null = null;
databaseStore.subscribe((v) => (database = v));

type MapStore = {
    inst: Map,

    arcLayer: LayerGroup,
    markerLayer: LayerGroup,
};

// a global map store representing the state of the map
export const map = (() => {
    const { subscribe, update, set } = writable<MapStore | null>(null);

    const init = (container: HTMLDivElement) => update(() => initImpl(container));
    const deinit = () => update(deinitImpl);

    const addIp = (ip: string) => update((store) => {
        if (store) addIpImpl(store, ip);
        return store;
    });

    return {
        subscribe,
        update,
        set,

        init,
        deinit,

        addIp,
    };
})();

// creates a MapStore on a div element
const initImpl = (container: HTMLDivElement): MapStore => {
    const inst = mkMap(container, { preferCanvas: false, minZoom: 2, maxZoom: 12 });
    inst.setView([30, 0], 2);
    inst.setMaxBounds(inst.getBounds());

    tileLayer
        .provider("OpenStreetMap.Mapnik", { noWrap: true })
        .addTo(inst);
    inst.setActiveArea(container); // from "leaflet-active-area", typescript doesn't recognize it.

    return {
        inst,
        arcLayer: layerGroup(),
        markerLayer: layerGroup(),
    }
};

// cleans up the map by removing it from the div
const deinitImpl = (store: MapStore | null): null => {
    if (store) {
        store.inst.remove();
    }

    return null;
};

const addIpImpl = async (store: MapStore, ip: string) => {
    const location = await lookupIp(ip, database);

    if (!location) {
        console.warn("couldn't find location for " + ip);
        return;
    }

    console.info(location);
};
