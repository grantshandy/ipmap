import daisyui from "daisyui";
import { lightTheme, darkTheme } from "./src/themes.json";

/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{svelte,ts}", "index.html"],
  theme: {
    extend: {},
  },
  plugins: [daisyui],
  daisyui: {
    themes: [lightTheme, darkTheme],
  },
};
