import { Channel } from "@tauri-apps/api/core";
import { captureError } from ".";
import { commands, events, type ActiveConnections, type ConnectionInfo, type Device, type GlobalPcapStateInfo } from "./raw";

type PcapStore = {
    status: {
        version: string,
        devices: Device[],
        capture: Device | null,
    } | string | null,
    connections: { [ip: string]: ConnectionInfo }
};

// let startCalled = false;

export let state: PcapStore = $state({ status: null, connections: {} });

const updatePcapState = (n: GlobalPcapStateInfo) => {
    if ("Loaded" in n) {
        // This is triggered if the capture is running, but
        // the page was reloaded so we don't have access to
        // the channel anymore. Just stop the capture.
        // TODO: is there a way to recover this?
        // if (n.Loaded.capture && !startCalled) {
        //     stopCapture();
        // }

        state.status = n.Loaded;
    } else {
        state.status = n.Unavailable;
    }
};

// update every time event fired from backend
const onConnectionRecv = (connections: ActiveConnections) => {
    state.connections = connections.data as { [ip: string]: ConnectionInfo };
};

// update once on page load
commands.syncPcapState().then((d) => {
    if (d.status == "ok") {
        updatePcapState(d.data);
    } else {
        state.status = d.status;
    }
})

events.pcapStateChange.listen((ev) => updatePcapState(ev.payload));

export const startCapture = (device: Device | null) => {
    if (device == null) return;

    // startCalled = true;
    captureError(commands.startCapture(device, new Channel(onConnectionRecv)));
};

export const stopCapture = () => captureError(commands.stopCapture());

window.onclose = () => commands.stopCapture();