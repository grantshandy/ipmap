import {
  events,
  commands,
  type DbStateInfo,
  type DbSetInfo,
  type DatabaseSource,
} from "./raw";

import * as dialog from "@tauri-apps/plugin-dialog";
import { displayError } from "./error";
import { Channel } from "@tauri-apps/api/core";

class Database implements DbStateInfo {
  ipv4: DbSetInfo = $state({ loaded: [], selected: null });
  ipv6: DbSetInfo = $state({ loaded: [], selected: null });
  combined: DbSetInfo = $state({ loaded: [], selected: null });

  loading: { name: string | null; progress: number | null } | null =
    $state(null);

  combinedEnabled: boolean = $derived(this.combined.selected != null);
  ipv4Enabled: boolean = $derived(this.ipv4.selected != null);
  ipv6Enabled: boolean = $derived(this.ipv6.selected != null);

  anyEnabled: boolean = $derived(
    this.ipv4Enabled || this.ipv6Enabled || this.combinedEnabled,
  );

  constructor() {
    commands.databaseState().then(this.update);
    events.dbStateChange.listen((ev) => this.update(ev.payload));
  }

  private update = (state: DbStateInfo) => {
    this.ipv4 = state.ipv4;
    this.ipv6 = state.ipv6;
    this.combined = state.combined;
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
  setSelected = (name: string | null | undefined) => {
    if (name) commands.setSelectedDatabase(name);
  };

  /**
   * Unload the database, freeing up memory.
   */
  unload = (name: string | null) => {
    if (name) commands.unloadDatabase(name);
  };

  /**
   * Lookup a given IP address in the currently selected database(s).
   */
  lookupIp = commands.lookupIp;

  /**
   * Get a hostname with the system for a given IP address.
   */
  lookupDns = commands.lookupDns;

  /**
   * Get a hostname with the system for a given IP address.
   */
  lookupHost = commands.lookupHost;

  /**
   * Attempt to get the user's current location
   */
  myLocation = commands.myLocation;
}

export default new Database();
