{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    utils.url = "github:numtide/flake-utils";
    rust = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, utils, rust, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust) ];
        };
      in
      {
        devShells.default = pkgs.mkShell {

          LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ pkgs.libpcap ]}:$LD_LIBRARY_PATH";

          buildInputs = with pkgs; [
            rust-bin.stable.latest.default
            rust-analyzer
            cargo-expand

            pnpm
            nodejs-slim

            webkitgtk_4_1
            pkg-config
            openssl
            libpcap
            xdg-utils
          ];
        };

        formatter = pkgs.nixpkgs-fmt;
      });
}
