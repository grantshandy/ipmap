import { writable } from "svelte/store";
import { event } from "@tauri-apps/api";
import { defaultDarkTheme, defaultLightTheme } from "../themes.json";
import type { ThemeState } from "../bindings";

const osPrefersDarkMode =
  window.matchMedia &&
  window.matchMedia("(prefers-color-scheme: dark)").matches;

export const theme = (() => {
  let local: ThemeState | null = null;

  const fromStorage = localStorage.getItem("theme");
  if (fromStorage) local = JSON.parse(fromStorage);

  const store = writable<ThemeState>(
    local ?? {
      dark: defaultDarkTheme,
      light: defaultLightTheme,
      isLight: osPrefersDarkMode,
    },
  );

  store.subscribe((theme) => {
    localStorage.setItem("theme", JSON.stringify(theme));
    event.emit("theme-change", theme);
  });

  event.listen("set-light-theme", (p) =>
    store.update((s) => {
      s.light = p.payload as string;
      return s;
    }),
  );

  event.listen("set-dark-theme", (p) =>
    store.update((s) => {
      s.dark = p.payload as string;
      return s;
    }),
  );

  const toggle = () =>
    store.update((store) => {
      store.isLight = !store.isLight;
      return store;
    });

  return {
    toggle,
    ...store,
  };
})();
