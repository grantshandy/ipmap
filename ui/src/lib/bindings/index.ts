import { message } from "@tauri-apps/plugin-dialog";
import { commands, type Result } from "./raw";

export type * from "./raw";

export const openAboutWindow = commands.openAboutWindow;

export { APP_VERSION, PLATFORM } from "./raw";

export const captureError = async <
  T,
  F extends (...args: any[]) => Promise<Result<T, string>>,
>(
  f: F,
  ...args: Parameters<F>
): Promise<T | null> => {
  try {
    const r = await f(...args);

    if (r.status === "error") {
      displayError(r.error);
      return null;
    } else {
      return r.data;
    }
  } catch (error) {
    displayError(`An unexpected error occurred: ${error}`);
    return null;
  }
};

export const displayError = (messageText: string) => {
  console.error(messageText);
  message(messageText, { title: "Ipmap Error", kind: "error" });
};
