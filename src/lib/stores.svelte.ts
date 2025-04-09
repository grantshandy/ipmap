import { message } from "@tauri-apps/plugin-dialog";
import { commands, events, type Device, type GlobalDatabaseStateInfo, type GlobalPcapStateInfo, type Result } from "../bindings";

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

export let pcapState: { version: string; devices: Device[], capturing: Device | null, error: string | null }
    = $state({ version: "", devices: [], capturing: null, error: "Loading..." });

const updatePcapState = (state: GlobalPcapStateInfo) => {
    console.log(state);

    if ("Loaded" in state) {
        pcapState.version = state.Loaded.version;
        pcapState.devices = state.Loaded.devices;
        pcapState.capturing = state.Loaded.capturing;
        pcapState.error = null;
    } else {
        pcapState.error = state.Unavailable;
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
