import { message } from "@tauri-apps/plugin-dialog";
import { type Result } from "./raw";

export type * from "./raw";
export * as cap from "./capture.svelte";
export * as db from "./database.svelte";

export const captureError = async <T>(f: Promise<Result<T, string>>): Promise<T | null> => {
    const r = await f;

    if (r.status == "error") {
        message(r.error, { title: "Ipmap Error", kind: "error" });
        console.error(r.error);
        return null;
    } else {
        return r.data;
    }
};
