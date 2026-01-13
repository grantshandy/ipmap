import {
  events,
  commands,
  type DbStateInfo,
  type DbSetInfo,
  type DatabaseSource,
} from "./bindings";

import * as dialog from "@tauri-apps/plugin-dialog";
import { Channel } from "@tauri-apps/api/core";

export type * from "./bindings";

const displayError = (messageText: string) => {
  console.error(messageText);
  dialog.message(messageText, { title: "Database Error", kind: "error" });
};

type LoadingState = { name: string | null; progress: number | null } | null;

class Database implements DbStateInfo {
  ipv4: DbSetInfo = $state({ loaded: [], selected: null });
  ipv6: DbSetInfo = $state({ loaded: [], selected: null });
  combined: DbSetInfo = $state({ loaded: [], selected: null });

  loading: LoadingState | null = $state(null);

  combinedEnabled: boolean = $derived(this.combined.selected != null);
  ipv4Enabled: boolean = $derived(this.ipv4.selected != null);
  ipv6Enabled: boolean = $derived(this.ipv6.selected != null);

  anyEnabled: boolean = $derived(
    this.ipv4Enabled || this.ipv6Enabled || this.combinedEnabled,
  );

  // If we've gotten a response from the backend yet.
  responseBack: boolean = $state(false);

  constructor() {
    commands
      .refreshCache()
      .then((ev) =>
        ev.status == "ok" ? this.update(ev.data) : displayError(ev.error),
      );
    events.dbStateChange.listen((ev) => this.update(ev.payload));
  }

  private update = (state: DbStateInfo) => {
    this.ipv4 = state.ipv4;
    this.ipv6 = state.ipv6;
    this.combined = state.combined;
    if (!this.responseBack) this.responseBack = true;
  };

  /**
   * Download or load a database from a file/url.
   *
   * Stores it in the user-wide compressed disk cache for fast access.
   *
   * @param source
   */
  downloadSource = async (source: DatabaseSource) => {
    this.loading = {
      name: null,
      progress: null,
    };

    const res = await commands.downloadSource(
      source,
      new Channel((name: string) => this.loading && (this.loading.name = name)),
      new Channel((p: number) => this.loading && (this.loading.progress = p)),
    );

    if (res.status == "error") {
      displayError(res.error);
    }

    this.loading = null;
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

    this.downloadSource({ file });
  };

  /**
   * Set the given database as the selected database for lookups.
   */
  setSelected = (name: DatabaseSource | null | undefined) => {
    if (name) commands.setSelectedDatabase(name);
  };

  /**
   * Unload the database, freeing up memory.
   */
  unload = (name: DatabaseSource | null) => {
    if (name) commands.unloadDatabase(name);
  };

  lookupIp = commands.lookupIp;
  lookupDns = commands.lookupDns;
  lookupHost = commands.lookupHost;
}

export default new Database();
