import { writable } from "svelte/store";
import { lightTheme, darkTheme } from "../themes";

const LS_KEY = "theme";

const osPrefersDarkMode = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
const storageInitValue: string | null = (() => {
    const v = localStorage.getItem(LS_KEY);
    localStorage.removeItem(LS_KEY);

    if (!v || (v != lightTheme && v != darkTheme)) return null;

    return v;
})();

export const theme = (() => {
    const { subscribe, update, set } = writable<string>(storageInitValue ?? (osPrefersDarkMode ? darkTheme : lightTheme));

    subscribe((theme) => localStorage.setItem(LS_KEY, theme));

    const toggle = () => update((theme) => {
        if (theme === lightTheme) {
            return darkTheme;
        } else {
            return lightTheme;
        }
    });

    const isLight = (): boolean => {
        let light = false;

        update((theme) => {
            light = (theme == lightTheme);
            return theme;
        });

        return light;
    }

    return {
        subscribe,
        update,
        set,
        toggle,
        isLight,
    };
})();

export { lightTheme, darkTheme };
