import { Channel } from "@tauri-apps/api/core";
import type { Event, UnlistenFn } from "@tauri-apps/api/event";
import {
  CAPTURE_CONNECTION_TIMEOUT,
  CAPTURE_REPORT_FREQUENCY,
  displayError,
  printError,
} from ".";
import {
  commands,
  events,
  type CaptureLocation,
  type CaptureLocations,
  type Connection,
  type Device,
  type Error,
  type PcapStateChange,
  type PcapStateInfo,
  type Result,
} from "./raw";

let startCalled = false;

type CoordKey = string;

type OnUpdate = (crd: CoordKey, loc: CaptureLocation) => void;
type OnIpChanged = (crd: CoordKey, loc: CaptureLocation | null) => void;
type OnStopping = () => void;

export class CaptureSession {
  updates: OnUpdate;
  ipChanged: OnIpChanged;
  stopping: OnStopping;

  connections: { [latlng: string]: CaptureLocation } = $state({});
  notFound: { [ip: string]: Connection } = $state({});
  session: Connection = $state({
    up: { avgS: 0, total: 0 },
    down: { avgS: 0, total: 0 },
  });
  maxThroughput = $state(0);

  constructor(updates: OnUpdate, ipChanged: OnIpChanged, stopping: OnStopping) {
    this.updates = updates;
    this.ipChanged = ipChanged;
    this.stopping = stopping;
  }

  /** Runs when the capture channel returns updates. Fires events. */
  update = (conns: CaptureLocations) => {
    this.maxThroughput = conns.maxThroughput;
    this.session = conns.session;
    this.connections = conns.updates as { [crd: string]: CaptureLocation };

    // Fire stopping, Pcap.stopCapture should clean us up after it returns.
    if (conns.last) {
      this.stopping();
      return;
    }

    for (const key of conns.connectionsChanged) {
      if (key in this.connections) {
        this.ipChanged(key, this.connections[key]);
      } else {
        this.ipChanged(key, null);
      }
    }

    for (const [crd, record] of Object.entries(this.connections)) {
      this.updates(crd, record);
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
  public startCapture = (
    updates: OnUpdate,
    ipChanged: OnIpChanged,
    stopping: OnStopping,
  ): CaptureSession | null => {
    if (this.device == null) return null;

    startCalled = true;

    const captureSession = new CaptureSession(updates, ipChanged, stopping);

    commands
      .startCapture(
        {
          device: this.device,
          connectionTimeout: CAPTURE_CONNECTION_TIMEOUT,
          reportFrequency: CAPTURE_REPORT_FREQUENCY,
        },
        new Channel(captureSession.update),
      )
      .then((res) => {
        if (res.status == "error") {
          printError(res.error).then(displayError);
        }
      });

    return captureSession;
  };

  /** Stop the current packet capture, if capturing. */
  public stopCapture = async () => {
    const r = await commands.stopCapture();
    if (r.status == "error") printError(r.error).then(displayError);
  };

  /** Runs when the backend fires the pcap update state event */
  private updateState = (state: PcapStateInfo | Event<PcapStateChange>) => {
    let info: PcapStateInfo;

    if ("payload" in state) {
      if (state.payload.status == "Ok") {
        info = state.payload;
      } else {
        printError(state.payload).then(displayError);
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
