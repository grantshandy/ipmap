import { writable } from "svelte/store";
import { geoip, type DatabaseInfo, type DatabaseQuery } from "../bindings";
import { open } from "@tauri-apps/api/dialog";
import { basename } from "@tauri-apps/api/path";

type DatabaseStore = {
  ipv4: DatabaseInfo | null;
  ipv6: DatabaseInfo | null;
  ipv4dbs: DatabaseInfo[];
  ipv6dbs: DatabaseInfo[];
  loading: string | null;
};

export const database = (() => {
  const { subscribe, update, set } = writable<DatabaseStore>({
    ipv4: null,
    ipv6: null,
    ipv4dbs: [],
    ipv6dbs: [],
    loading: null,
  });

  const query = (): DatabaseQuery => {
    let v: DatabaseQuery = {
      ipv4: null,
      ipv6: null,
    };

    update((db) => {
      if (db.ipv4) v.ipv4 = db.ipv4.query;
      if (db.ipv6) v.ipv6 = db.ipv6.query;

      return db;
    });

    return v;
  };

  const startLoading = (text: string) =>
    update((store) => {
      store.loading = text;
      return store;
    });

  const stopLoading = () =>
    update((store) => {
      store.loading = null;
      return store;
    });

  const updateListings = () =>
    geoip.listDatabases().then((dbs) =>
      update((_) => {
        const ipv4dbs = dbs.filter((v) => v.kind == "IPv4");
        const ipv6dbs = dbs.filter((v) => v.kind == "IPv6");

        return {
          ipv4dbs,
          ipv6dbs,
          ipv4: ipv4dbs.length > 0 ? ipv4dbs[0] : null,
          ipv6: ipv6dbs.length > 0 ? ipv6dbs[0] : null,
          loading: null,
        };
      }),
    );

  const importDatabase = async () => {
    const dir = await open({
      multiple: false,
      filters: [
        {
          name: "IP City CSV Database",
          extensions: ["csv"],
        },
      ],
    });
    if (!dir) return;

    startLoading(await basename(dir as string));
    await geoip.loadDatabase(dir as string).catch(() => null);
    stopLoading();
    await updateListings();
  };

  const unloadDatabase = async (info: DatabaseInfo) => {
    if (info.query == "Internal") return;

    await geoip.unloadDatabase(info.query.Loaded);
    await updateListings();
  };

  (async () => {
    startLoading("Internal Databases");
    await updateListings();
  })();

  return {
    subscribe,
    update,
    set,

    query,

    startLoading,
    stopLoading,

    importDatabase,
    unloadDatabase,
  };
})();
