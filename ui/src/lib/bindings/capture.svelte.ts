import { Channel } from "@tauri-apps/api/core";
import type { Event, UnlistenFn } from "@tauri-apps/api/event";
import {
  CAPTURE_CONNECTION_TIMEOUT,
  CAPTURE_REPORT_FREQUENCY,
  captureError,
  captureErrorBasic,
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

type CaptureState = {
  version: string;
  devices: Device[];
  capture: Device | null;
};

type ConnectionStart = (ip: string, info: ConnectionInfo) => void;
type ConnectionEnd = (ip: string) => void;
type ConnectionUpdate = (ip: string, info: ConnectionInfo) => void;

export class Pcap {
  startCalled = false;
  device: Device | null = $state(null);
  status: PcapStateInfo = $state({
    version: "",
    devices: [],
    capture: null,
  });
  connections: { [ip: string]: ConnectionInfo } = $state({});

  public unlisten!: UnlistenFn;

  private connStarts: ConnectionStart[] = [];
  private connEnds: ConnectionEnd[] = [];
  private connUpdate: ConnectionUpdate[] = [];

  constructor(status: CaptureState) {
    console.log("capture binding initialized");

    this.update(status);
    events.pcapStateChange.listen(this.update).then((u) => (this.unlisten = u));
  }

  private onConnectionRecv = (conns: Connections) => {
    const connUpdates = conns.updates as { [ip: string]: ConnectionInfo };

    if (conns.stopping) {
      for (const ip of Object.keys(this.connections)) {
        this.fireConnEnd(ip);
      }

      this.connections = {};
      return;
    }

    for (const [ip, data] of Object.entries(connUpdates)) {
      this.connections[ip] = data;
    }

    // if (conns.started.length > 0) console.log(conns.started.length, "connections added");
    // if (conns.ended.length > 0) console.log(conns.ended.length, "connections ended");

    for (const ip of conns.started) {
      // console.log(`${ip} started`);
      this.fireConnStart(ip, this.connections[ip]);
    }

    for (const ip of conns.ended) {
      // console.log(`${ip} ended`);
      this.fireConnEnd(ip);
      delete this.connections[ip];
    }

    for (const [ip, data] of Object.entries(this.connections)) {
      this.fireConnUpdate(ip, data);
    }
  };

  public startCapture = () => {
    if (this.device == null) return;

    this.startCalled = true;

    const channel = new Channel(this.onConnectionRecv);

    captureError(
      commands.startCapture(
        {
          device: this.device,
          connectionTimeout: CAPTURE_CONNECTION_TIMEOUT,
          reportFrequency: CAPTURE_REPORT_FREQUENCY,
        },
        channel,
      ),
    );
  };

  public stopCapture = () => captureErrorBasic(commands.stopCapture());

  private update = (state: PcapStateInfo | Event<PcapStateChange>) => {
    let info: PcapStateInfo;

    if ("payload" in state) {
      if (state.payload.status == "Ok") {
        info = state.payload;
      } else {
        displayError(printError(state.payload));
        return;
      }
    } else {
      info = state;
    }

    console.log("new pcap update", info);

    this.status = info;

    if (this.status.capture != null && !this.startCalled) {
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

  onStart = (l: ConnectionStart) => this.connStarts.push(l);
  onEnd = (l: ConnectionEnd) => this.connEnds.push(l);
  onUpdate = (l: ConnectionUpdate) => this.connUpdate.push(l);

  fireConnStart = (ip: string, info: ConnectionInfo) =>
    this.connStarts.forEach((cb) => cb(ip, info));
  fireConnEnd = (ip: string) => this.connEnds.forEach((cb) => cb(ip));
  fireConnUpdate = (ip: string, info: ConnectionInfo) =>
    this.connUpdate.forEach((cb) => cb(ip, info));
}

export const newPcapInstance = (): Promise<Result<Pcap, Error>> =>
  commands
    .initPcap()
    .then((p) =>
      p.status == "ok"
        ? { status: "ok", data: new Pcap(p.data) }
        : { status: "error", error: p.error },
    );
