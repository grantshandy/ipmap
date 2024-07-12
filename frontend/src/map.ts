import { divIcon, type DivIcon, type LatLngExpression } from "leaflet";

export const DEFAULT_POS: LatLngExpression = [25, 0];
export const DEFAULT_ZOOM = 2;

export const mkIcon = (count: number | null, active?: boolean): DivIcon =>
  divIcon({
    html: `<div class="marker-icon ${active ? "bg-primary" : "bg-secondary"}">${count ? count : ""}</div>`,
    className: "dummyclass",
    iconSize: [20, 20],
    iconAnchor: [10, 10],
  });

