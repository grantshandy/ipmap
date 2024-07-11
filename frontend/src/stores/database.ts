import { writable } from "svelte/store";
import type { DatabaseInfo } from "../bindings";

export const database = (() => {
    const { subscribe, update, set } = writable<DatabaseInfo | null>(null);

    return {
        subscribe,
        update,
        set,
        path: (): string | null => {
            let name: string | null = null;

            update((db) => {
                if (db) name = db.path;
                return db;
            });

            return name;
        }
    };
})();
