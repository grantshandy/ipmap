import {
  events,
  commands,
  type DatabaseStoreInfo,
  type BuiltinDatabaseSources,
} from "./raw";
import * as dialog from "@tauri-apps/plugin-dialog";
import { captureError } from ".";

export interface SourceInfo {
  kind: BuiltinDatabaseSources;
}

class DatabaseStore implements DatabaseStoreInfo {
  loading: boolean = $state(false);
  loaded: string[] = $state([]);
  selected: string | null = $state(null);
  anyEnabled: boolean = $derived(this.selected != null);

  constructor() {
    commands.databaseInfo().then(this.update);
    events.databaseStoreInfo.listen((ev) => this.update(ev.payload));
  }

  private update = (state: DatabaseStoreInfo) => {
    this.loading = state.loading;
    this.loaded = state.loaded;
    this.selected = state.selected;
  };

  open = async () => {
    if (this.loading) return;

    const file = await dialog.open({
      title: "Open IP Geolocation City Database",
      multiple: false,
      directory: false,
      filters: [
        {
          name: "IP Geolocation City Database",
          extensions: ["csv", "csv.gz", "mmdb"],
        },
      ],
    });

    if (!file) return;

    console.log("opening database", file);
    commands.loadFile(file);
  };

  download = commands.download;

  setSelected = (name: string | null | undefined) => {
    if (name) commands.setSelected(name);
  };

  unload = (name: string | null) => {
    // if (name) captureError(commands.unloadDatabase, name);
  };

  lookupIp = commands.lookupIp;
  lookupDns = commands.lookupDns;
  lookupHost = commands.lookupHost;
  initCache = commands.initCache;
  // myLocation = commands.myLocation;
}

export default new DatabaseStore();
