{
  description = "rebooted";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat.url = "github:edolstra/flake-compat";
    naersk.url = "github:nix-community/naersk/master";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      naersk,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        name = "rebooted";

        devShells.default =
          with pkgs;
          pkgs.mkShell {
            nativeBuildInputs = [
              pkg-config
            ];
            buildInputs = [
              cargo
              rustc
              rustfmt
              rustPackages.clippy
              cargo-watch
              nixfmt
              just
            ];
          };

        packages.default = naersk-lib.buildPackage ./.;
      }
    );
}
