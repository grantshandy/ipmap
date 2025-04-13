import { commands, events, type ConnectionInfo, type Device, type GlobalPcapStateInfo } from "./raw";

type PcapStore = {
    state: {
        version: string,
        devices: Device[],
        capture: Device | null
    } | string | null,
    connections: {
        active: ConnectionInfo[],
        all: ConnectionInfo[]
    } | null
};

export let pcap: PcapStore = $state({ state: null, connections: null });

export const refreshConnections = async () => {
    const all = await commands.allConnections();
    
    if (all == null) {
        pcap.connections == null;
    } else {
        if (!pcap.connections) {
            pcap.connections = { all: [], active: [] };
        }

        pcap.connections.all = all;
    }
};

const updatePcapState = (state: GlobalPcapStateInfo) => {
    refreshConnections();

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
commands.pcapState().then(updatePcapState);

// update every time event fired from backend
events.pcapStateChange.listen((ev) => updatePcapState(ev.payload));

// update active connections when fired
events.activeConnections.listen((ev) => {
    if (!pcap.connections) pcap.connections = { all: [], active: [] };

    pcap.connections.active = ev.payload;
});
