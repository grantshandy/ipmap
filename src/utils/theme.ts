import { writable } from "svelte/store";
import { event } from "@tauri-apps/api";
import { defaultDarkTheme, defaultLightTheme } from "../themes.json";
import type { Event } from "@tauri-apps/api/event";

const osPrefersDarkMode =
  window.matchMedia &&
  window.matchMedia("(prefers-color-scheme: dark)").matches;

export const theme = (() => {
  const { set, subscribe, update } = writable<string>(
    localStorage.getItem("theme") ??
      (osPrefersDarkMode ? defaultDarkTheme : defaultLightTheme),
  );

  event.listen("set-theme", (p: Event<string>) => update((_) => p.payload));
  subscribe((theme) => localStorage.setItem("theme", theme));

  return {
    set,
    subscribe,
    update,
  };
})();
