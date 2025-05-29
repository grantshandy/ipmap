# use with nixpkgs 24.05 or later.

{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  RUST_LOG = "debug";
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [ pkgs.libpcap ]}:$LD_LIBRARY_PATH";

  buildInputs = with pkgs; [
    cargo
    rustfmt
    cargo-expand
    cargo-watch
    cargo-tauri

    pnpm
    nodejs-slim

    webkitgtk_4_1
    pkg-config
    openssl
    libpcap
    xdg-utils
  ];
}