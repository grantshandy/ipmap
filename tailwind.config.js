import daisyui from "daisyui";
import { lightThemes, darkThemes } from "./src/themes.json";

/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{svelte,ts,svg}", "*.html"],
  theme: {
    extend: {},
  },
  plugins: [daisyui],
  daisyui: {
    themes: [...lightThemes, ...darkThemes],
  },
};
