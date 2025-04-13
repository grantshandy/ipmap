import { message } from "@tauri-apps/plugin-dialog";
import { commands, events, type ConnectionInfo, type Device, type GlobalDatabaseStateInfo, type GlobalPcapStateInfo, type Result } from "../bindings";

export let dbState: GlobalDatabaseStateInfo = $state({
    ipv4: { loaded: [], selected: null },
    ipv6: { loaded: [], selected: null },
    loading: null,
});

const updateDbState = (state: GlobalDatabaseStateInfo) => {
    dbState.loading = state.loading;

    dbState.ipv4.loaded = state.ipv4.loaded;
    dbState.ipv4.selected = state.ipv4.selected
        ? dbState.ipv4.loaded.filter(
            (info) => info.path == state.ipv4.selected?.path,
        )[0]
        : null;

    dbState.ipv6.loaded = state.ipv6.loaded;
    dbState.ipv6.selected = state.ipv6.selected
        ? dbState.ipv6.loaded.filter(
            (info) => info.path == state.ipv6.selected?.path,
        )[0]
        : null;
};

// initialize the state and keep it up to date from the backend.
commands.databaseState().then(updateDbState);
events.databaseStateChange.listen((ev) => updateDbState(ev.payload));

export let connections: { active: ConnectionInfo[], all: ConnectionInfo[] } = $state({ active: [], all: [] });
events.activeConnections.listen((ev) => {
    connections.active = ev.payload;
});

export const refreshConnections = () => commands
    .allConnections()
    .then((r) => {
        if (r != null) connections.all = r;
    });

export let pcap: { state: { version: string, devices: Device[], capture: Device | null } | string | null }
    = $state({ state: null });

const updatePcapState = (state: GlobalPcapStateInfo) => {
    console.log(state);

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

commands.pcapState().then(updatePcapState);
events.pcapStateChange.listen((ev) => updatePcapState(ev.payload));

export let captureError = async <T>(f: Promise<Result<T, string>>): Promise<T | null> => {
    const r = await f;

    if (r.status == "error") {
        message(r.error, { title: "Ipmap Error", kind: "error" });
        console.error(r.error);
        return null;
    } else {
        return r.data;
    }
}
