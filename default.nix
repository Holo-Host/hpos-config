{ pkgs ? import ./pkgs.nix {} }:

with pkgs;

let
  inherit (rust.packages.nightly) rustPlatform;
in

{
  holo-configure = buildRustPackage rustPlatform {
    name = "holo-configure";
    src = gitignoreSource ./.;
    cargoDir = ".";

    meta.platforms = lib.platforms.all;
  };
}
