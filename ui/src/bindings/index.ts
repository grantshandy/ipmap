import { message } from "@tauri-apps/plugin-dialog";
import { type Result } from "./raw";

import database from "./database.svelte";
export { database };

export * from "./capture.svelte";

export type * from "./raw";

export const captureError = async <T>(
  f: Promise<Result<T, string>>,
): Promise<T | null> => {
  const r = await f;

  if (r.status == "error") {
    displayError(r.error);
    return null;
  } else {
    return r.data;
  }
};

export const displayError = (messageText: string) => {
  console.error(messageText);
  message(messageText, { title: "Ipmap Error", kind: "error" });
};
