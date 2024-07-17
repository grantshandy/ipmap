import { divIcon, Marker, type DivIcon, type LatLngExpression } from "leaflet";
import type { Coordinate } from "./bindings";

export const DEFAULT_POS: LatLngExpression = [25, 0];
export const DEFAULT_ZOOM = 2;

export const mkIcon = (count: number | null, active?: boolean): DivIcon =>
  divIcon({
    html: `<div class="${active ? "marker-icon-active" : "marker-icon"}">${count ? count : ""}</div>`,
    className: "dummyclass",
    iconSize: active ? [30, 30] : [20, 20],
    iconAnchor: active ? [15, 15] : [10, 10],
  });

export type CaptureLocation = {
  coord: Coordinate;
  marker: Marker;
  ips: Set<string>;
};
