import { commands, events, type GlobalDatabaseStateInfo } from "./raw";
import { open } from "@tauri-apps/plugin-dialog";
import { captureError } from "./index";

export let db: GlobalDatabaseStateInfo = $state({
    ipv4: { loaded: [], selected: null },
    ipv6: { loaded: [], selected: null },
    loading: null,
});

const updateDbState = (state: GlobalDatabaseStateInfo) => {
    db.loading = state.loading;

    db.ipv4.loaded = state.ipv4.loaded;
    db.ipv4.selected = state.ipv4.selected
        ? db.ipv4.loaded.filter(
            (info) => info.path == state.ipv4.selected?.path,
        )[0]
        : null;

    db.ipv6.loaded = state.ipv6.loaded;
    db.ipv6.selected = state.ipv6.selected
        ? db.ipv6.loaded.filter(
            (info) => info.path == state.ipv6.selected?.path,
        )[0]
        : null;
};

// initialize the state and keep it up to date from the backend.
commands.databaseState().then(updateDbState);
events.databaseStateChange.listen((ev) => updateDbState(ev.payload));

export const openDatabaseDialog = async () => {
    const file = await open({
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

    if (file && !db.loading) {
        captureError(commands.loadDatabase(file));
    };
};