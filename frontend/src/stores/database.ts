import { writable } from "svelte/store";
import type { DatabaseInfo } from "../bindings";

export const database = writable<DatabaseInfo | null>(null);
