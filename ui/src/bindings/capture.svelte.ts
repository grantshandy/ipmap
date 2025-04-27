import { Channel } from "@tauri-apps/api/core";
import { captureError } from ".";
import { commands, events, type ActiveConnections, type ConnectionInfo, type Device, type GlobalPcapStateInfo } from "./raw";

type PcapStore = {
    state: {
        version: string,
        devices: Device[],
        capture: Device | null,
    } | string | null,
    connections: { [ip: string]: ConnectionInfo }
};

let startCalled = false;

export let pcap: PcapStore = $state({ state: null, connections: {} });

const updatePcapState = (state: GlobalPcapStateInfo) => {
    if ("Loaded" in state) {
        // This is triggered if the capture is running, but
        // the page was reloaded so we don't have access to
        // the channel anymore. Just stop the capture.
        // TODO: is there a way to recover this?
        if (state.Loaded.capture && !startCalled) {
            stopCapture();
        }

        pcap.state = {
            version: state.Loaded.version,
            devices: state.Loaded.devices,
            capture: state.Loaded.capture,
        };
    } else {
        pcap.state = state.Unavailable;
    }
};

// update once on page load
commands.pcapState().then((d) => {
    if (d.status == "ok") {
        updatePcapState(d.data);
    } else {
        pcap.state = d.status;
    }
});

// update every time event fired from backend
events.pcapStateChange.listen((ev) => updatePcapState(ev.payload));

const onConnectionRecv = (connections: ActiveConnections) => {
    pcap.connections = connections.data as { [ip: string]: ConnectionInfo };
}

export const startCapture = (device: Device | null) => {
    if (device == null) return;

    startCalled = true;
    captureError(
        commands.startCapture(
            device,
            new Channel<ActiveConnections>(onConnectionRecv)
        )
    );
};

export const stopCapture = () => captureError(commands.stopCapture());
