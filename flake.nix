{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
  };
  outputs = { self, nixpkgs, flake-utils, cargo2nix }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ cargo2nix.overlays.default ];
          };
          rustPkgs = pkgs.rustBuilder.makePackageSet {
            rustVersion = "latest";
            packageFun = import ./Cargo.nix;
          };
        in
        rec
        {
          packages = {
            nasty = (rustPkgs.workspace.nasty { });
            default = packages.nasty;

          };
          devShells.default = with pkgs; mkShell {
            buildInputs = [ rust-bin.stable.latest.default openssl pkg-config jaq ];
          };
        }

      );
}
