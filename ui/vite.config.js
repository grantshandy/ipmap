import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { viteStaticCopy } from "vite-plugin-static-copy";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    sveltekit(),
    tailwindcss(),
    viteStaticCopy({
      targets: [
        {
          src: "node_modules/@openglobus/og/lib/res",
          dest: ".",
        },
      ],
    }),
  ],

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/crates/**"],
    },

    fs: {
      allow: [
        "../crates/tauri-plugin-ipgeo",
        "../crates/tauri-plugin-pcap",
        "../crates/desktop",
      ],
    },
  },
}));
