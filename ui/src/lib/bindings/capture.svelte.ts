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
  type ConnectionInfo,
  type Connections,
  type Device,
  type Error,
  type PcapStateChange,
  type PcapStateInfo,
  type Result,
} from "./raw";

let startCalled = false;

export class Pcap {
  /** The currently selected device for capturing. */
  device: Device | null = $state(null);

  /** Current status of the capture state on the backend. */
  status: PcapStateInfo = $state({
    version: "",
    devices: [],
    capture: null,
  });

  /** Active connections the computer is currently engaged in. */
  connections: { [ip: string]: ConnectionInfo } = $state({});

  /** Call this to stop listening to the backend */
  public unlisten!: UnlistenFn;

  /** An event that is triggered when a connection with a given IP address starts */
  public start: EventDispatcher<ConnectionStart> = new EventDispatcher();

  /** An event that is triggered when a connection with a given IP address ends */
  public end: EventDispatcher<ConnectionEnd> = new EventDispatcher();

  /** An event that is triggered when the backend reports new data for the connection */
  public update: EventDispatcher<ConnectionUpdate> = new EventDispatcher();

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
  public startCapture = async () => {
    if (this.device == null) return;

    startCalled = true;

    const channel = new Channel(this.updateConnections);

    const res = await commands.startCapture(
      {
        device: this.device,
        connectionTimeout: CAPTURE_CONNECTION_TIMEOUT,
        reportFrequency: CAPTURE_REPORT_FREQUENCY,
      },
      channel,
    );

    if (res.status == "error") {
      displayError(await printError(res.error));
    }
  };

  /** Stop the current packet capture, if capturing. */
  public stopCapture = () =>
    commands.stopCapture().then((r) => {
      if (r.status == "error") printError(r.error).then(displayError);
    });

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

  /** Runs when the capture channel returns updates. Fires events. */
  private updateConnections = (conns: Connections) => {
    const connUpdates = conns.updates as { [ip: string]: ConnectionInfo };

    if (conns.stopping) {
      for (const ip of Object.keys(this.connections)) {
        this.end.fire(ip);
      }

      this.connections = {};
      return;
    }

    for (const [ip, data] of Object.entries(connUpdates)) {
      this.connections[ip] = data;
    }

    for (const ip of conns.started) {
      this.start.fire(ip, this.connections[ip]);
    }

    for (const ip of conns.ended) {
      this.end.fire(ip);
      delete this.connections[ip];
    }

    for (const [ip, data] of Object.entries(this.connections)) {
      this.update.fire(ip, data);
    }
  };
}

/** A generic event dispatcher for connection events */
class EventDispatcher<T extends (...args: any[]) => void> {
  private handlers: T[] = [];

  constructor() {}

  /** Register an event handler */
  public on = (l: T) => this.handlers.push(l);

  /** Dispatch this event */
  public fire = (...args: Parameters<T>) =>
    this.handlers.forEach((handler) => handler(...args));
}

type ConnectionStart = (ip: string, info: ConnectionInfo) => void;
type ConnectionEnd = (ip: string) => void;
type ConnectionUpdate = (ip: string, info: ConnectionInfo) => void;
