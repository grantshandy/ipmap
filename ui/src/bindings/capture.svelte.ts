import { Channel } from "@tauri-apps/api/core";
import { captureError } from ".";
import { commands, events, type Connections, type ConnectionInfo, type Device, type PcapStateInfo } from "./raw";

type CaptureState = {
    version: string,
    devices: Device[],
    capture: Device | null,
};

type ConnectionStart = (ip: string, info: ConnectionInfo) => void;
type ConnectionEnd = (ip: string) => void;

class Pcap {
    startCalled = false;
    device: Device | null = $state(null);
    status: CaptureState | string | null = $state(null);
    connections: { [ip: string]: ConnectionInfo } = $state({});

    private connStarts: ConnectionStart[] = [];
    private connEnds: ConnectionEnd[] = [];

    constructor() {
        console.log("capture binding initialized");

        // initialize state from backend on page load
        commands
            .syncPcapState()
            // flatten result :)
            .then((result) => result.status == "ok" ? result.data : { Unavailable: result.status })
            .then(this.update);

        events
            .pcapStateChange
            .listen((ev) => this.update(ev.payload));
    }

    private update = (state: PcapStateInfo) => {
        if ("Unavailable" in state) {
            this.status = state.Unavailable;
            return;
        }

        this.status = state.Loaded;

        if (this.status.capture != null && !this.startCalled) {
            console.warn("stopping previous page-load capture session");
            captureError(commands.stopCapture());
        }

        // **this.device must be a reference to a device in the status.devices array**
        // because of the obj equivalence check in the device <select>

        if (this.device == null) {
            this.device = this.status.devices[0];
        } else {
            this.device =
                this.status.devices.find((d) => d.name == this.device?.name)
                ?? null;
        }

        if (this.status.capture) {
            const captureName = this.status.capture.name;

            this.device =
                this.status.devices.find((d) => d.name == captureName)
                ?? null;
        }
    }

    public onConnStart = (l: ConnectionStart) => this.connStarts.push(l);
    public onConnEnd = (l: ConnectionEnd) => this.connEnds.push(l);

    private fireConnStart = (ip: string, info: ConnectionInfo) =>
        this.connStarts.forEach((cb) => cb(ip, info));

    private fireConnEnd = (ip: string) =>
        this.connEnds.forEach((cb) => cb(ip));

    private onConnectionRecv = (conns: Connections) => {
        const connUpdates = conns.updates as { [ip: string]: ConnectionInfo };

        if (conns.stopping_capture) {
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
            this.fireConnStart(ip, this.connections[ip]);
        }

        for (const ip of conns.ended) {
            this.fireConnEnd(ip);
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
}

export default new Pcap();
