import {
  map as mkMap,
  type Map,
  tileLayer,
  LayerGroup,
  layerGroup,
  Marker,
  marker,
  DivIcon,
  divIcon,
  type LatLngExpression,
} from "leaflet";
import "leaflet-edgebuffer";
import "leaflet-active-area";

import { writable, type Subscriber, type Unsubscriber, type Updater } from "svelte/store";
import {
  geoip,
  type ConnectionDirection,
  type ConnectionInfo,
  type Coordinate,
} from "../bindings";
import { GeodesicLine } from "leaflet.geodesic";

const SELECT_ZOOM = 7;

export type IpLocation = {
  coord: Coordinate;
  marker: Marker;
  ips: Set<string>;
};

export type MapState = {
  inst: Map;
  ips: Set<string>;
  arcLayer: LayerGroup;
  markerLayer: LayerGroup;
  selection: IpLocation | null;
  locations: { [id: LocationKey]: IpLocation };
  currentConnections: {
    [id: string]: { info: ConnectionInfo; arc: GeodesicLine };
  };
  locationMarker: Marker | null;
};

export type MapStore = {
  subscribe(this: void, run: Subscriber<MapState | null>): Unsubscriber,
  update: (this: void, updater: Updater<MapState>) => void,
  set: (v: MapState) => void,

  setSelection: (selection: IpLocation) => void,
  setArcState: (state: ConnectionInfo[]) => void,
  addIp: (ip: string) => void,
  invalidateSize: () => void,
  resetView: () => void,
};

/** a global map store representing the state of the map */
export const createMap = (container: HTMLDivElement): MapStore => {
  const { subscribe, update, set } = writable<MapState>(initImpl(container));

  const setSelection = (selection: IpLocation) =>
    update((store) => {
      if (store) setSelectionImpl(store, selection);
      return store;
    });

  const setArcState = (state: ConnectionInfo[]) =>
    update((store) => {
      if (store) setArcStateImpl(store, state);
      return store;
    });

  const addIp = (ip: string) =>
    update((store) => {
      if (store) addIpImpl(store, ip);
      return store;
    });

  const invalidateSize = () =>
    update((store) => {
      store.inst.invalidateSize();
      return store;
    });

  const resetView = () =>
    update((store) => {
      if (store) resetMapView(store.inst);
      return store;
    });

  return {
    subscribe,
    update,
    set,
    setSelection,
    setArcState,
    addIp,
    invalidateSize,
    resetView,
  };
};

// creates a MapStore on a div element
const initImpl = (container: HTMLDivElement): MapState => {
  const arcLayer = layerGroup();
  const markerLayer = layerGroup();

  const inst = mkMap(container, {
    preferCanvas: false,
    minZoom: 2,
    maxZoom: 13,
    zoomControl: false,
    layers: [arcLayer, markerLayer],
  });
  resetMapView(inst);
  tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
    attribution: "&copy; OSM Contributors",
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

const resetMarkersImpl = (store: MapState) => {
  store.markerLayer.eachLayer((l) => l.remove());
  store.locations = {};
};

const setSelectionImpl = (store: MapState, selection: IpLocation) => {
  if (store.inst.getZoom() < SELECT_ZOOM) {
    store.inst.flyTo(selection.coord, SELECT_ZOOM);
  } else {
    store.inst.panTo(selection.coord);
  }
  store.selection = selection;
};

const setArcStateImpl = (store: MapState, newState: ConnectionInfo[]) => {
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

  geoip.myLocation().then((from) => {
    // add arcs that don't already exist
    for (const newState of Object.values(newStates)) {
      // discard already existing connections
      if (store.currentConnections[newState.ip]) continue;

      geoip.lookupIp(newState.ip).then((to) => {
        if (!to) return;

        store.currentConnections[newState.ip] = {
          arc: mkLine(from, to, newState.direction, store.arcLayer),
          info: newState,
        };
      });
    }
  });
};

const addIpImpl = async (store: MapState, ip: string) => {
  if (store.ips.has(ip)) return;
  store.ips.add(ip);

  const coord = await geoip.lookupIp(ip);

  if (!coord) return;

  const key = mkLocationKey(coord);

  const iploc = store.locations[key];

  if (iploc) {
    iploc.ips.add(ip);
    iploc.marker.setIcon(mkIcon(iploc.ips.size, false));
  } else {
    store.locations[key] = {
      coord,
      ips: new Set([ip]),
      marker: marker(coord, { icon: mkIcon(1, false) }).addTo(
        store.markerLayer,
      ),
    };
  }
};

const addLocationMarkerIfNotExists = (store: MapState) => {
  if (!store.locationMarker) {
    store.locationMarker = marker([0, 0], {
      icon: mkIcon(null, true),
    });

    geoip.myLocation().then((location) => {
      if (!store.locationMarker) return;

      store.locationMarker.setLatLng(location).addTo(store.markerLayer);
    });
  }
};

const resetMapView = (map: Map) => {
  map.setView([25, 0], 2);
};

export const mkIcon = (count: number | null, active?: boolean): DivIcon =>
  divIcon({
    html: `<div class="marker-icon ${active ? "bg-primary" : "bg-secondary"}">${count ? count : ""}</div>`,
    className: "dummyclass",
    iconSize: [20, 20],
    iconAnchor: [10, 10],
  });

type LocationKey = string;
const mkLocationKey = (loc: Coordinate) => `${loc.lat}${loc.lng}`;

const mkLine = (
  current: LatLngExpression,
  to: LatLngExpression,
  direction: ConnectionDirection,
  map: LayerGroup<any>,
) => {
  const line = new GeodesicLine([current, to], {
    weight: 2,
    steps: 3,
    opacity: 0.5,
    className: direction,
  }).addTo(map);

  return line;
};
