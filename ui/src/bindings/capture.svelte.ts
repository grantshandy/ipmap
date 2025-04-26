import { commands, events, type ConnectionInfo, type Device, type GlobalPcapStateInfo } from "./raw";

type PcapStore = {
    state: {
        version: string,
        devices: Device[],
        capture: Device | null
    } | string | null,
    connections: { [ip: string]: ConnectionInfo }
};

export let pcap: PcapStore = $state({ state: null, connections: {} });

const updatePcapState = (state: GlobalPcapStateInfo) => {
    if ("Loaded" in state) {
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
    }
});

// update every time event fired from backend
events.pcapStateChange.listen((ev) => updatePcapState(ev.payload));

// update active connections when fired
events.activeConnections.listen((ev) => {
    pcap.connections = ev.payload as { [ip: string]: ConnectionInfo };
});
