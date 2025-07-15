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
      useMold = {
        stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
      };

      package = {
        stdenv,
        lib,
        pnpm_10,
        nodejs,
        rustPlatform,
        cargo-tauri,
        pkg-config,
        wrapGAppsHook4,
        openssl,
        dbus,
        gdk-pixbuf,
        glib,
        gobject-introspection,
        gtk3,
        libsoup_3,
        libayatana-appindicator,
        webkitgtk_4_1,
        zenity,
        ...
      }: let
        pnpm = pnpm_10;
      in
        rustPlatform.buildRustPackage rec {
          pname = "ipmap";
          version = "5.0.0";

          src = ./.;

          pnpmRoot = "ui";
          pnpmDeps = pnpm.fetchDeps {
            inherit pname version src;
            postPatch = "cd ${pnpmRoot}";
            hash = "sha256-MWWe4NDg32jQySQCZ2KMCkVHXQrmLTEumQmcCnGHnOg=";
          };

          buildType = "debug";
          doCheck = false;
          useFetchCargoVendor = true;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [
            cargo-tauri.hook
            pkg-config
            nodejs
            pnpm.configHook
            wrapGAppsHook4
          ];

          buildInputs = [
            openssl
            dbus
            gdk-pixbuf
            glib
            gobject-introspection
            gtk3
            libsoup_3
            webkitgtk_4_1
          ];

          preFixup = ''
            gappsWrapperArgs+=(
              # Otherwise blank screen, see https://github.com/tauri-apps/tauri/issues/9304
              --set-default WEBKIT_DISABLE_DMABUF_RENDERER 1
              --prefix PATH ":" ${lib.makeBinPath [zenity]}
            )
          '';
        };
    in {
      packages.default = pkgs.callPackage package {};

      devShells.default = pkgs.mkShell.override useMold {
        RUST_LOG = "debug";
        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
        LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [pkgs.libpcap]}:$LD_LIBRARY_PATH";
        WEBKIT_DISABLE_DMABUF_RENDERER = "1";
        DB_PRELOADS = "/home/grant/Documents/ipdbs/dbip-city-ipv4.csv.gz:/home/grant/Documents/ipdbs/dbip-city-ipv6.csv.gz";

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
    });
}
