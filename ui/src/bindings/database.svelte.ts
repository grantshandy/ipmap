import { commands, events, type DatabaseInfo, type DatabaseStateInfo, type GlobalDatabaseStateInfo } from "./raw";
import * as dialog from "@tauri-apps/plugin-dialog";
import { Channel } from "@tauri-apps/api/core";

export const ipv4: DatabaseStateInfo = $state({ loaded: [], selected: null });
export const ipv6: DatabaseStateInfo = $state({ loaded: [], selected: null });
export const loading: { msg: string | null } = $state({ msg: null });

const updateDbState = (state: GlobalDatabaseStateInfo) => {
    loading.msg = state.loading;

    ipv4.loaded = state.ipv4.loaded;
    ipv4.selected
        ? ipv4.loaded.find((info) => info.path === ipv4.selected?.path)
        : null;

    ipv6.loaded = state.ipv6.loaded;
    ipv6.selected
        ? ipv6.loaded.find((info) => info.path === ipv6.selected?.path)
        : null;
};

// initialize the state and keep it up to date from the backend.
commands.databaseState().then(updateDbState);
events.databaseStateChange.listen((ev) => updateDbState(ev.payload));

export const openDatabaseDialog = async () => {
    const file = await dialog.open({
        title: "Open IP Geolocation City Database",
        multiple: false,
        directory: false,
        filters: [
            {
                name: "IP Geolocation City Database",
                extensions: ["csv", "csv.gz"],
            },
        ],
    });

    console.log("opening database", file);

    if (file && !loading.msg) {
        commands.loadDatabase(
            file,
            new Channel((msg: string) => dialog.message(msg, { title: "Ipmap Error", kind: "error" }))
        )
    }
};

export const setSelectedDatabase = commands.setSelectedDatabase;

export const unloadSelectedDatabase = (isIpv6: boolean) => {
    const selected = isIpv6 ? ipv6.selected : ipv4.selected;

    if (selected) {
        commands.unloadDatabase(selected);
    }
};

export const lookupIp = commands.lookupIp;
