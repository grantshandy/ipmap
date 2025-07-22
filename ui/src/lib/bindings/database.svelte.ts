import {
  events,
  commands,
  type DbStateInfo,
  type DbCollectionInfo,
} from "./raw";
import * as dialog from "@tauri-apps/plugin-dialog";
import { captureError } from ".";

class Database implements DbStateInfo {
  ipv4: DbCollectionInfo = $state({ loaded: [], selected: null });
  ipv6: DbCollectionInfo = $state({ loaded: [], selected: null });
  loading: string | null = $state(null);

  ipv4Enabled: boolean = $derived(this.ipv4.selected != null);
  ipv6Enabled: boolean = $derived(this.ipv6.selected != null);

  anyEnabled: boolean = $derived(this.ipv4Enabled || this.ipv6Enabled);

  constructor() {
    commands.databaseState().then(this.update);
    events.dbStateChange.listen((ev) => this.update(ev.payload));
  }

  private update = (state: DbStateInfo) => {
    this.loading = state.loading;
    this.ipv4 = state.ipv4;
    this.ipv6 = state.ipv6;
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
          extensions: ["csv", "csv.gz"],
        },
      ],
    });

    if (!file) return;

    console.log("opening database", file);
    commands.loadDatabase(file);
  };

  setSelected = (name: string | null | undefined) => {
    if (name) commands.setSelectedDatabase(name);
  };

  unload = (name: string | null) => {
    if (name) captureError(commands.unloadDatabase, name);
  };

  lookupIp = commands.lookupIp;
  lookupDns = commands.lookupDns;
  lookupHost = commands.lookupHost;
  loadInternals = commands.loadInternals;
  myLocation = commands.myLocation;
}

export default new Database();
