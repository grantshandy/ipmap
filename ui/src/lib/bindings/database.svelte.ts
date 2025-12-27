import {
  events,
  commands,
  type DbStateInfo,
  type DbSetInfo,
  type Result,
} from "./raw";
import * as dialog from "@tauri-apps/plugin-dialog";
import { captureError, displayError } from "./error";
import { Channel } from "@tauri-apps/api/core";

class Database implements DbStateInfo {
  ipv4: DbSetInfo = $state({ loaded: [], selected: null });
  ipv6: DbSetInfo = $state({ loaded: [], selected: null });

  ipv4Enabled: boolean = $derived(this.ipv4.selected != null);
  ipv6Enabled: boolean = $derived(this.ipv6.selected != null);
  loading: { name: string | null; progress: number | null } | null =
    $state(null);

  anyEnabled: boolean = $derived(this.ipv4Enabled || this.ipv6Enabled);

  constructor() {
    commands.databaseState().then(this.update);
    events.dbStateChange.listen((ev) => this.update(ev.payload));
  }

  private update = (state: DbStateInfo) => {
    this.ipv4 = state.ipv4;
    this.ipv6 = state.ipv6;

    console.log(state);
  };

  openFile = async () => {
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

    this.loading = {
      name: null,
      progress: null,
    };

    commands
      .downloadSource(
        { file },
        new Channel(
          (name: string) => this.loading && (this.loading.name = name),
        ),
        new Channel((p: number) => {
          console.log(p);
          return this.loading && (this.loading.progress = p);
        }),
      )
      .then((r) => {
        if (r.status == "error") {
          displayError(r.error);
        }

        this.loading = null;
      });
  };

  setSelected = (name: string | null | undefined) => {
    if (name) commands.setSelectedDatabase(name);
  };

  unload = (name: string | null) => {
    if (name) commands.unloadDatabase(name);
  };

  lookupIp = commands.lookupIp;
  lookupDns = commands.lookupDns;
  lookupHost = commands.lookupHost;
  myLocation = commands.myLocation;
}

export default new Database();
