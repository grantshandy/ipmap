{
  stdenv,
  lib,
  pnpm_9,
  nodejs,
  rustPlatform,
  cargo-tauri,
  pkg-config,
  wrapGAppsHook3,
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
  version = "5.0.0";

  frontend = stdenv.mkDerivation rec {
    pname = "ipmap-frontend";
    src = ./ui;
    inherit version;

    nativeBuildInputs = [nodejs pnpm_9.configHook];

    pnpmDeps = pnpm_9.fetchDeps {
      inherit pname version src;
      hash = "sha256-VTXS1ENa7t891h0I3nchNCnGzNq+Sumj/VjRcFy79eA=";
    };

    # The build phase, which now uses the pre-fetched dependencies.
    buildPhase = ''
      runHook preBuild
      pnpm run build
      runHook postBuild
    '';

    # The install phase to copy the final build artifacts.
    installPhase = ''
      runHook preInstall
      mkdir -p $out
      cp -r build/* $out/
      runHook postInstall
    '';
  };
in
  rustPlatform.buildRustPackage rec {
    pname = "ipmap";
    inherit version;

    src = lib.cleanSourceWith {
      src = ./.;
      filter = name: _: name != "ui";
    };

    doCheck = false;
    useFetchCargoVendor = true;
    cargoLock.lockFile = ./Cargo.lock;

    installTargets = [ "ipmap" "ipmap-child" ];

    nativeBuildInputs = [
      cargo-tauri.hook
      pkg-config
      wrapGAppsHook3
    ];

    buildInputs = [
      openssl
      dbus
      gdk-pixbuf
      glib
      gobject-introspection
      gtk3
      libsoup_3
      libayatana-appindicator
      webkitgtk_4_1
    ];

    postPatch = ''
      mkdir -p ui/
      ln -s ${frontend} ui/build

      substituteInPlace crates/desktop/tauri.conf.json \
        --replace-fail "pnpm build" ""
    '';

    preFixup = ''
      gappsWrapperArgs+=(
        # Otherwise blank screen, see https://github.com/tauri-apps/tauri/issues/9304
        --set WEBKIT_DISABLE_DMABUF_RENDERER 1
        --prefix PATH ":" ${lib.makeBinPath [ zenity ]}
        --prefix LD_LIBRARY_PATH ":" ${lib.makeLibraryPath [ libayatana-appindicator ]}
      )
    '';
  }
