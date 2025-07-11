{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    utils,
    ...
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};

      shell =
        pkgs.mkShell.override {
          stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
        } {
          RUST_LOG = "debug";
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [pkgs.libpcap]}:$LD_LIBRARY_PATH";

          DB_PRELOADS = "/home/grant/Documents/ipdbs/dbip-city-ipv4.csv.gz";

          shellHook = ''
            export IPMAP_CHILD="$(pwd)/target/release/ipmap-child"
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

            pnpm
            nodejs-slim

            webkitgtk_4_1
            pkg-config
            openssl
            libpcap
            xdg-utils
            hyperfine
          ];
        };
    in {
      devShells.default = shell;
      packages.default = ipmap;
    });
}
