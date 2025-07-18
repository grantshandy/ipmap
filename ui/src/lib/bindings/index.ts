import { message } from "@tauri-apps/plugin-dialog";
import {
  commands,
  type Coordinate,
  type Device,
  type Duration,
  type Error,
  type ErrorKind,
  type Location,
  type ThroughputTrackerInfo,
  type Result,
  type ConnectionInfo,
} from "./raw";
import { GeodesicLine } from "leaflet.geodesic";
import { divIcon, geodesic, Marker } from "leaflet";

import database from "./database.svelte";
export { database };

export * from "./capture.svelte";

export type * from "./raw";

export const traceroute = {
  run: commands.runTraceroute,
  enabled: commands.tracerouteEnabled,
};

export const utils = {
  openAboutWindow: commands.openAboutWindow,
  version: commands.version,
  platform: commands.platform,
};

export const isError = (value: unknown): value is Error => {
  if (typeof value !== "object" || value === null) return false;

  if (!("kind" in value) || !("message" in value)) return false;
  if (typeof value.kind !== "string") return false;

  const validKinds: ErrorKind[] = [
    "UnexpectedType",
    "TerminatedUnexpectedly",
    "Ipc",
    "InsufficientPermissions",
    "LibLoading",
    "Runtime",
    "ChildNotFound",
    "EstablishConnection",
    "Io",
  ];

  if (!validKinds.includes(value.kind as ErrorKind)) return false;
  if (typeof value.message !== "string" && value.message !== null) return false;

  return true;
};

export const printError = commands.printError;

export const captureError = async <
  T,
  F extends (...args: any[]) => Promise<Result<T, string>>,
>(
  f: F,
  ...args: Parameters<F>
): Promise<T | null> => {
  try {
    const r = await f(...args);

    if (r.status === "error") {
      displayError(r.error);
      return null;
    } else {
      return r.data;
    }
  } catch (error) {
    displayError(`An unexpected error occurred: ${error}`);
    return null;
  }
};

export const displayError = (messageText: string) => {
  console.error(messageText);
  message(messageText, { title: "Ipmap Error", kind: "error" });
};

export const durationFromMillis = (milliseconds: number): Duration => {
  const ONE_SECOND_IN_MILLIS = 1000;
  const ONE_MILLI_IN_NANOS = 1_000_000; // 1 million nanoseconds in a millisecond

  const secs = Math.floor(milliseconds / ONE_SECOND_IN_MILLIS);
  const remainingMillis = milliseconds % ONE_SECOND_IN_MILLIS;
  const nanos = remainingMillis * ONE_MILLI_IN_NANOS;

  return {
    secs,
    nanos,
  };
};

// TODO: move to settings window
export const CAPTURE_CONNECTION_TIMEOUT: Duration = { secs: 1, nanos: 0 };
export const CAPTURE_REPORT_FREQUENCY: Duration = durationFromMillis(150);
export const CAPTURE_SHOW_ARCS = true;
export const CAPTURE_COLORS = true;
export const CAPTURE_VARY_SIZE = true;

export const newArc = (
  from: Coordinate,
  to: Coordinate,
  info: ConnectionInfo,
  maxThroughput: number,
): GeodesicLine => {
  const up = info.up.avgS;
  const down = info.down.avgS;

  const { weight, opacity } = CAPTURE_VARY_SIZE
    ? calculateWeights(up + down, maxThroughput)
    : { weight: 2, opacity: 0.8 };

  return geodesic([from, to], {
    steps: 5,
    // TODO: default styles other than basic blue?
    className: CAPTURE_COLORS ? connectionDirection(up, down) : "",
    weight,
    opacity,
  });
};

export const updateArcStyle = (
  record: ActiveLocationRecord,
  maxThroughput: number,
) => {
  const [up, down] = recordThroughputs(record);

  if (CAPTURE_COLORS) {
    updateArcDirection(record.arc, up, down);
  }

  if (CAPTURE_VARY_SIZE) {
    record.arc.setStyle(calculateWeights(up + down, maxThroughput));
  }
};

const ARC_MIN_OPACITY = 0.25;
const ARC_MAX_OPACITY = 1.0;
const ARC_MIN_WEIGHT = 1.5;
const ARC_MAX_WEIGHT = 6;

const calculateWeights = (throughput: number, maxThroughput: number) => ({
  weight: lerp(throughput, 0, maxThroughput, ARC_MIN_WEIGHT, ARC_MAX_WEIGHT),
  opacity: lerp(throughput, 0, maxThroughput, ARC_MIN_OPACITY, ARC_MAX_OPACITY),
});

const updateArcDirection = (arc: GeodesicLine, up: number, down: number) => {
  const direction = connectionDirection(up, down);

  const svgElement = arc.getElement();
  if (svgElement) {
    svgElement.setAttribute("class", `leaflet-interactive ${direction}`);
  }
};

export const recordThroughputs = (
  record: ActiveLocationRecord,
): [number, number] => {
  const values = Object.values(record.ips);

  return [
    values.map((p) => p.up.avgS).reduce((a, b) => a + b, 0),
    values.map((p) => p.down.avgS).reduce((a, b) => a + b, 0),
  ];
};

/** Corresponds with class names path.up, path.down, path.mixed */
export type ConnectionDirection = "up" | "down" | "mixed";

export const connectionDirection = (up: number, down: number) => {
  const ratio = Math.min(up, down) / Math.max(up, down);

  if (ratio > 0.7) {
    return "mixed";
  } else if (up > down) {
    return "up";
  } else {
    return "down";
  }
};

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

export type CoordKey = string;
export type ActiveLocationRecord = {
  crd: CoordKey;
  marker: Marker;
  arc: GeodesicLine;
  ips: Record<string, ConnectionInfo>;
  location: Location;
};

export const coordKey = (p: Coordinate): CoordKey => `${p.lat}${p.lng}`;

export const updateIcon = (
  loc: ActiveLocationRecord,
  focused: ActiveLocationRecord | null,
) => {
  const active = focused && focused.crd == loc.crd;

  const iconSize = active ? 30 : 20;
  const iconAnchor = iconSize / 2;

  loc.marker.setIcon(
    divIcon({
      html: `<span>${Object.keys(loc.ips).length}</span>`,
      className: active ? "marker-icon-active" : "marker-icon",
      iconSize: [iconSize, iconSize],
      iconAnchor: [iconAnchor, iconAnchor],
    }),
  );
};

export const renderDeviceName = async (device: Device): Promise<string> => {
  if ((await utils.platform()) == "windows") {
    return device.description ?? device.name;
  } else {
    return `${device.name}${device.description ? ": (" + device.description + ")" : ""}`;
  }
};

export const humanFileSize = (size: number) => {
  const i = size == 0 ? 0 : Math.floor(Math.log(size) / Math.log(1024));
  return (
    +(size / Math.pow(1024, i)).toFixed(2) * 1 +
    " " +
    ["B", "kB", "MB", "GB", "TB"][i]
  );
};

export const movingAverageInfo = (info: ThroughputTrackerInfo): string =>
  `${humanFileSize(info.total)} | ${humanFileSize(info.avgS)}/s`;

export const regionNames = new Intl.DisplayNames(["en"], { type: "region" });

export const renderLocationName = (l: Location) =>
  `${l.city ?? "Unknown City"}${l.region ? `, ${l.region}` : ""}, ${regionNames.of(l.countryCode)}`;
