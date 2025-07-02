# use with nixpkgs 25.05 or later.
{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell.override {
  stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
} {
  RUST_LOG = "debug";
  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [pkgs.libpcap]}:$LD_LIBRARY_PATH";

  shellHook = ''
    export PCAP_CHILD="$(pwd)/target/release/pcap-child"
  '';

  buildInputs = with pkgs; [
    cargo
    rustc
    rustfmt
    cargo-expand
    cargo-watch
    cargo-tauri
    cargo-bloat
    clippy

    rust-analyzer
    typescript-language-server
    svelte-language-server

    pnpm
    nodejs-slim

    webkitgtk_4_1
    pkg-config
    openssl
    libpcap
    xdg-utils
  ];
}
