import { Channel } from "@tauri-apps/api/core";
import type { Event, UnlistenFn } from "@tauri-apps/api/event";

import {
  PCAP_ERROR_KINDS,
  commands,
  events,
  type Duration,
  type CaptureLocation,
  type CaptureLocations,
  type Connection,
  type Device,
  type Error,
  type PcapStateChange,
  type PcapStateInfo,
  type Result,
} from "./bindings";

export * from "./bindings";
export type * from "./bindings";

import * as dialog from "@tauri-apps/plugin-dialog";

const displayError = (messageText: string) => {
  console.error(messageText);
  dialog.message(messageText, { title: "Database Error", kind: "error" });
};

export const traceroute = {
  run: commands.runTraceroute,
  enabled: commands.tracerouteEnabled,
};

export const printError = commands.printError;

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
export const CAPTURE_REPORT_FREQUENCY: Duration = durationFromMillis(100);

export const isError = (value: unknown): value is Error =>
  value !== null &&
  typeof value === "object" &&
  (!("message" in value) || typeof value.message === "string") &&
  "kind" in value &&
  PCAP_ERROR_KINDS.includes(value.kind as any);

let startCalled = false;

type CoordKey = string;

export type SessionCallbacks = {
  stopping?: () => void;
  locationAdded?: (crd: CoordKey, loc: CaptureLocation) => void;
  locationRemoved?: (crd: CoordKey) => void;
  update?: (crd: CoordKey, loc: CaptureLocation) => void;
};

export class CaptureSession {
  connections: { [crd: string]: CaptureLocation } = $state({});
  session: Connection = $state({
    up: { avgS: 0, total: 0 },
    down: { avgS: 0, total: 0 },
  });
  maxThroughput = $state(0);

  notFound: { [ip: string]: Connection } = $state({});
  notFoundCount = $derived(Object.keys(this.notFound).length);

  cb: SessionCallbacks;

  constructor(callbacks: SessionCallbacks) {
    this.cb = callbacks;
  }

  stop = () => {
    for (const key of Object.keys(this.connections)) {
      this.cb.locationRemoved?.(key);
    }

    this.cb.stopping?.();
  };

  /** Runs when the capture channel returns updates. Fires events. */
  update = (conns: CaptureLocations) => {
    // Fire stopping, Pcap.stopCapture should clean us up after it returns.
    if (conns.last) {
      this.stop();
      return;
    }

    this.maxThroughput = conns.maxThroughput;
    this.session = conns.session;
    this.connections = conns.updates as { [crd: string]: CaptureLocation };
    this.notFound = conns.notFound as { [ip: string]: Connection };

    for (const key of conns.started) {
      this.cb.locationAdded?.(key, this.connections[key]);
    }

    for (const key of conns.ended) {
      this.cb.locationRemoved?.(key);
    }

    for (const [crd, record] of Object.entries(this.connections)) {
      this.cb.update?.(crd, record);
    }
  };
}

export class Pcap {
  /** The currently selected device for capturing. */
  device: Device | null = $state(null);

  /** Current status of the capture state on the backend. */
  status: PcapStateInfo = $state({
    version: "",
    devices: [],
    capture: null,
  });

  public capture: CaptureSession | null = $state(null);

  /** Call this to stop listening to the backend */
  public unlisten!: UnlistenFn;

  /** Initialize a new Pcap instance */
  public static init = (): Promise<Result<Pcap, Error>> =>
    commands
      .initPcap()
      .then((p) =>
        p.status == "ok"
          ? { status: "ok", data: new Pcap(p.data) }
          : { status: "error", error: p.error },
      );

  /** Initialize with initial data and start listening to state change events. */
  constructor(status: PcapStateInfo) {
    console.log("capture binding initialized");

    this.updateState(status);
    events.pcapStateChange
      .listen(this.updateState)
      .then((u) => (this.unlisten = u));
  }

  /** Start capturing on this.device */
  public startCapture = (callbacks: SessionCallbacks) => {
    if (this.device == null || this.capture != null) return;

    startCalled = true;

    this.capture = new CaptureSession(callbacks);

    commands
      .startCapture(
        {
          device: this.device,
          connectionTimeout: CAPTURE_CONNECTION_TIMEOUT,
          reportFrequency: CAPTURE_REPORT_FREQUENCY,
        },
        new Channel(this.capture.update),
      )
      .then((res) => {
        if (res.status == "error") {
          commands.printError(res.error).then(displayError);
        }
      });
  };

  /** Stop the current packet capture, if capturing. */
  public stopCapture = async () => {
    const r = await commands.stopCapture();

    this.capture = null;

    if (r.status == "error") commands.printError(r.error).then(displayError);
  };

  /** Runs when the backend fires the pcap update state event */
  private updateState = (state: PcapStateInfo | Event<PcapStateChange>) => {
    let info: PcapStateInfo;

    if ("payload" in state) {
      if (state.payload.status == "Ok") {
        info = state.payload;
      } else {
        commands.printError(state.payload).then(displayError);
        return;
      }
    } else {
      info = state;
    }

    console.log("new pcap update", info);

    this.status = info;

    if (this.status.capture != null && !startCalled) {
      console.warn("stopping previous page-load capture session");
      this.stopCapture();
    }

    // **this.device must be a reference to a device in the status.devices array**
    // because of the obj equivalence check in the device <select>
    if (this.device == null) {
      this.device = this.status.devices[0];
    } else {
      this.device =
        this.status.devices.find((d) => d.name == this.device?.name) ?? null;
    }

    if (this.status.capture) {
      const captureName = this.status.capture.name;

      this.device =
        this.status.devices.find((d) => d.name == captureName) ?? null;
    }
  };
}
