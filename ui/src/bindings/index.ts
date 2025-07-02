import { message } from "@tauri-apps/plugin-dialog";
import {
  commands,
  type Duration,
  type Error,
  type Result,
  type TracerouteParams,
} from "./raw";

import database from "./database.svelte";
import type { Channel } from "@tauri-apps/api/core";
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

export const runTraceroute = (
  params: TracerouteParams,
  channel: Channel<number>,
) => captureError(commands.runTraceroute(params, channel));

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
export const CAPTURE_REPORT_FREQUENCY: Duration = durationFromMillis(300);
