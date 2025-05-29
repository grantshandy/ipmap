import { events, commands, type DbStateInfo, type DbCollectionInfo } from "./raw";
import * as dialog from "@tauri-apps/plugin-dialog";
import { Channel } from "@tauri-apps/api/core";
import { captureError, displayError } from ".";

class Database implements DbStateInfo {
    ipv4: DbCollectionInfo = $state({ loaded: [], selected: null });
    ipv6: DbCollectionInfo = $state({ loaded: [], selected: null });
    loading: string | null = $state(null);

    anyLoaded: boolean = $derived(this.ipv4.selected != null || this.ipv6.selected != null);

    constructor() {
        console.log("database binding initialized");

        commands.databaseState().then(this.update);
        events.dbStateChange.listen((ev) => this.update(ev.payload));
    }

    private update = (state: DbStateInfo) => {
        console.log("update from backend", state);

        this.loading = state.loading;
        this.ipv4 = state.ipv4;
        this.ipv6 = state.ipv6;
    }

    open = async () => {
        if (this.loading) return;

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

        if (!file) return;

        console.log("opening database", file);

        return captureError(commands.loadDatabase(file, new Channel(displayError)));
    }

    setSelected = (name: string | null | undefined) => {
        if (name) commands.setSelectedDatabase(name);
    }

    unload = (name: string | null) => {
        if (name) captureError(commands.unloadDatabase(name));
    };

    lookupIp = commands.lookupIp;
}

export default new Database();
