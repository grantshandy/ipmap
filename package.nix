{
  pnpm_
  rustPlatform
}: let
  version = "5.0.0";

  frontend = pkgs.stdenv.mkDerivation rec {
    pname = "ipmap-frontend";
    src = ./ui;
    inherit version;

    nativeBuildInputs = with pkgs; [nodejs pnpm.configHook];

    pnpmDeps = pkgs.pnpm.fetchDeps {
      inherit pname version src;
      hash = "sha256-MWWe4NDg32jQySQCZ2KMCkVHXQrmLTEumQmcCnGHnOg=";
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

    src = pkgs.lib.cleanSourceWith {
      src = ./.;
      filter = path: _: baseNameOf path != "ui";
    };

    doCheck = false;
    useFetchCargoVendor = true;
    cargoLock.lockFile = ./Cargo.lock;

    nativeBuildInputs = with pkgs; [
      cargo-tauri.hook
      pkg-config
      wrapGAppsHook3
    ];

    buildInputs = with pkgs; [
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
        --prefix PATH ":" ${
        pkgs.lib.makeBinPath [
          pkgs.zenity
        ]
      }
        --prefix LD_LIBRARY_PATH ":" ${
        pkgs.lib.makeLibraryPath [
          pkgs.libayatana-appindicator
        ]
      }
      )
    '';
  };
