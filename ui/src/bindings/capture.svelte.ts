import { Channel } from "@tauri-apps/api/core";
import { captureError } from ".";
import { commands, events, type ActiveConnections, type ConnectionInfo, type Device, type PcapStateInfo } from "./raw";

type CaptureState = {
    version: string,
    devices: Device[],
    capture: Device | null,
};

type ConnectionStart = (ip: string, info: ConnectionInfo) => void;
type ConnectionEnd = (ip: string) => void;

class Pcap {
    device: Device | null = $state(null);
    status: CaptureState | string | null = $state(null);
    connections: { [ip: string]: ConnectionInfo } = $state({});

    private connStarts: ConnectionStart[] = [];
    private connEnds: ConnectionEnd[] = [];

    constructor() {
        console.log("capture binding initialized");

        commands
            .syncPcapState()
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

    private onConnectionRecv = (conns: ActiveConnections) => {
        const newConns = conns.data as { [ip: string]: ConnectionInfo };

        // Add/update new connections
        for (const [ip, data] of Object.entries(newConns)) {
            if (!(ip in this.connections)) {
                this.fireConnStart(ip, data);
            }

            this.connections[ip] = data;
        }

        // Remove ended connections
        for (const ip of Object.keys(this.connections)) {
            if (!(ip in newConns)) {
                this.fireConnEnd(ip);
                delete this.connections[ip];
            }
        }
    };

    public startCapture = () => {
        if (this.device == null) return;

        const channel = new Channel(this.onConnectionRecv);

        captureError(commands.startCapture(this.device, channel));
    };

    public stopCapture = () => captureError(commands.stopCapture());
}

export default new Pcap();
