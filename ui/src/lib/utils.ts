import type { Location } from "tauri-plugin-ipgeo-api";

export const CAPTURE_SHOW_ARCS = true;
export const CAPTURE_SHOW_MARKERS = true;
export const CAPTURE_COLORS = true;
export const CAPTURE_VARY_SIZE = true;
export const CAPTURE_SHOW_NOT_FOUND = true;

export const regionNames = new Intl.DisplayNames(["en"], { type: "region" });

export const renderLocationName = (l: Location) =>
  `${l.city ?? "Unknown City"}${l.region ? `, ${l.region}` : ""}, ${regionNames.of(l.countryCode)}`;

export const lerp = (
  value: number,
  inMin: number,
  inMax: number,
  outMin: number,
  outMax: number,
): number => {
  // Clamp the value to ensure it's within the input range
  const clampedValue = Math.max(inMin, Math.min(value, inMax));
  return (
    ((clampedValue - inMin) * (outMax - outMin)) / (inMax - inMin) + outMin
  );
};
