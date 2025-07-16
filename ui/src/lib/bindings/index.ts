import { message } from "@tauri-apps/plugin-dialog";
import {
  commands,
  type Coordinate,
  type Device,
  type Duration,
  type Error,
  type ErrorKind,
  type Location,
  type MovingAverageInfo,
  type Result,
} from "./raw";
import { GeodesicLine } from "leaflet.geodesic";
import { geodesic } from "leaflet";

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

export const CAPTURE_CONNECTION_TIMEOUT: Duration = { secs: 1, nanos: 0 };
export const CAPTURE_REPORT_FREQUENCY: Duration = durationFromMillis(150);

export const renderDeviceName = async (device: Device): Promise<string> => {
  if ((await utils.platform()) == "windows") {
    return device.description ?? device.name;
  } else {
    return `${device.name}${device.description ? ": (" + device.description + ")" : ""}`;
  }
};

export type ConnectionDirection = "up" | "down" | "mixed";

export const calculateConnectionDirection = (
  up: number,
  down: number,
): ConnectionDirection => {
  const CUTOFF = 0.7;

  const ratio = Math.min(up, down) / Math.max(up, down);

  if (ratio > CUTOFF) {
    return "mixed";
  } else if (up > down) {
    return "up";
  } else {
    return "down";
  }
};

export const arcFromDirection = (
  from: Coordinate,
  to: Coordinate,
  direction: ConnectionDirection,
): GeodesicLine =>
  geodesic([from, to], {
    weight: 2,
    steps: 3,
    opacity: 0.5,
    className: direction,
  });

export const humanFileSize = (size: number) => {
  const i = size == 0 ? 0 : Math.floor(Math.log(size) / Math.log(1024));
  return (
    +(size / Math.pow(1024, i)).toFixed(2) * 1 +
    " " +
    ["B", "kB", "MB", "GB", "TB"][i]
  );
};

export const movingAverageInfo = (info: MovingAverageInfo): string =>
  `${humanFileSize(info.total)} | ${humanFileSize(info.avgS)}/s`;

export const regionNames = new Intl.DisplayNames(["en"], { type: "region" });

export const renderLocationName = (l: Location) =>
  `${l.city ?? "Unknown City"}${l.region ? `, ${l.region}` : ""}, ${regionNames.of(l.countryCode)}`;
