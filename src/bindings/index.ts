import { message } from "@tauri-apps/plugin-dialog";
import { type Result } from "./raw";

export * from "./raw";
export * from "./capture.svelte";
export * from "./database.svelte";

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
