/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{svelte,ts}", "index.html"],
  theme: {
    extend: {},
  },
  plugins: [require("daisyui")],
  daisyui: {
    themes: ["business"],
  },
}
