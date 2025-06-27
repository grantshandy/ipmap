import { Channel } from "@tauri-apps/api/core";
import { captureError } from ".";
import {
  commands,
  events,
  type Connections,
  type ConnectionInfo,
  type Device,
  type PcapStateInfo,
} from "./raw";

type CaptureState = {
  version: string;
  devices: Device[];
  capture: Device | null;
};

type ConnectionStart = (ip: string, info: ConnectionInfo) => void;
type ConnectionEnd = (ip: string) => void;

class ConnectionChangeEvents {
  private connStarts: ConnectionStart[] = [];
  private connEnds: ConnectionEnd[] = [];

  constructor() {}

  onStart = (l: ConnectionStart) => this.connStarts.push(l);
  onEnd = (l: ConnectionEnd) => this.connEnds.push(l);

  fireConnStart = (ip: string, info: ConnectionInfo) =>
    this.connStarts.forEach((cb) => cb(ip, info));

  fireConnEnd = (ip: string) => this.connEnds.forEach((cb) => cb(ip));
}

export class Pcap {
  startCalled = false;
  device: Device | null = $state(null);
  status: CaptureState = $state({
    version: "",
    devices: [],
    capture: null,
  });
  connections: { [ip: string]: ConnectionInfo } = $state({});

  conn = new ConnectionChangeEvents();

  constructor(status: CaptureState) {
    this.update(status);

    events.pcapStateChange.listen((ev) => this.update(ev.payload));

    console.log("capture binding initialized");
  }

  private onConnectionRecv = (conns: Connections) => {
    const connUpdates = conns.updates as { [ip: string]: ConnectionInfo };

    if (conns.stoppingCapture) {
      for (const ip of Object.keys(this.connections)) {
        this.conn.fireConnEnd(ip);
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
      this.conn.fireConnStart(ip, this.connections[ip]);
    }

    for (const ip of conns.ended) {
      // console.log(`${ip} ended`);
      this.conn.fireConnEnd(ip);
      delete this.connections[ip];
    }
  };

  public startCapture = () => {
    if (this.device == null) return;

    this.startCalled = true;

    const channel = new Channel(this.onConnectionRecv);

    captureError(commands.startCapture(this.device, channel));
  };

  public stopCapture = () => captureError(commands.stopCapture());

  private update = (state: PcapStateInfo) => {
    console.log("new pcap update", state);

    this.status = state;

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
}

export const newPcapInstance = async (): Promise<Pcap | string> =>
  commands
    .initPcap()
    .then((r) => (r.status === "ok" ? new Pcap(r.data) : r.error));
