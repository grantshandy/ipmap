import { message } from "@tauri-apps/plugin-dialog";
import {
  commands,
  type Coordinate,
  type Device,
  type Duration,
  type Error,
  type MovingAverageInfo,
  type Result,
} from "./raw";

import database from "./database.svelte";
import type { Channel } from "@tauri-apps/api/core";
import { GeodesicLine } from "leaflet.geodesic";
import { geodesic } from "leaflet";
export { database };

export * from "./capture.svelte";

export type * from "./raw";

export const isError = (err: any): err is Error =>
  err != null &&
  typeof err == "object" &&
  (err.t == "Runtime" ||
    err.t == "Ipc" ||
    err.t == "LibLoading" ||
    err.t == "InsufficientPermissions");

export const printError = (err: Error): string => {
  if (err.t == "Runtime") {
    return `Error in child process: ${err.c}`;
  } else if (err.t == "Ipc") {
    return `Failure to connect to child process: ${err.c}`;
  } else if (err.t == "LibLoading") {
    return `Unable to load libpcap: ${err.c}`;
  } else if (err.t == "InsufficientPermissions") {
    return "Insufficient permissions for the child process";
  } else {
    return "Unknown Error Type";
  }
};

export const captureErrorBasic = async <T>(
  f: Promise<Result<T, string>>,
): Promise<T | null> => {
  const r = await f;

  if (r.status == "error") {
    displayError(r.error);
    return null;
  } else {
    return r.data;
  }
};

export const captureError = async <T>(
  f: Promise<Result<T, Error>>,
): Promise<T | null> => {
  const r = await f;

  if (r.status == "error") {
    console.error(r.error);
    displayError(printError(r.error));
    return null;
  } else {
    return r.data;
  }
};

export const displayError = (messageText: string) => {
  console.error(messageText);
  message(messageText, { title: "Ipmap Error", kind: "error" });
};

export const platform = commands.platform;
export const isTracerouteEnabled = commands.tracerouteEnabled;

export const runTraceroute = commands.runTraceroute;

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

export const CAPTURE_CONNECTION_TIMEOUT: Duration = { secs: 5, nanos: 0 };
export const CAPTURE_REPORT_FREQUENCY: Duration = durationFromMillis(150);

export const renderDeviceName = async (device: Device): Promise<string> => {
  if ((await platform()) == "Windows") {
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
