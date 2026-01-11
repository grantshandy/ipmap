import { message } from "@tauri-apps/plugin-dialog";
import {
  commands,
  PCAP_ERROR_KINDS,
  PLATFORM as PLATFORM_CONST,
  type Device,
  type Duration,
  type Error,
  type Location,
  type Result,
  type Throughput,
} from "./raw";

export * from "./capture.svelte";

export type * from "./raw";

export const traceroute = {
  run: commands.runTraceroute,
  enabled: commands.tracerouteEnabled,
};

export const openAboutWindow = commands.openAboutWindow;

export { APP_VERSION } from "./raw";

export const PLATFORM: string = PLATFORM_CONST;

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
export const CAPTURE_REPORT_FREQUENCY: Duration = durationFromMillis(100);
export const CAPTURE_SHOW_ARCS = true;
export const CAPTURE_SHOW_MARKERS = true;
export const CAPTURE_COLORS = true;
export const CAPTURE_VARY_SIZE = true;
export const CAPTURE_SHOW_NOT_FOUND = true;

export const renderDeviceName = async (device: Device): Promise<string> => {
  if (PLATFORM === "windows") {
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

export const isError = (value: unknown): value is Error =>
  value !== null &&
  typeof value === "object" &&
  (!("message" in value) || typeof value.message === "string") &&
  "kind" in value &&
  PCAP_ERROR_KINDS.includes(value.kind as any);

export const throughputInfo = (info: Throughput): string =>
  `${humanFileSize(info.avgS)}/s | ${humanFileSize(info.total)}`;

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
