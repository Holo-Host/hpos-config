{ pkgs ? import ./pkgs.nix {} }:

with pkgs;

{
  holo-configure = buildRustPackage rustPlatform {
    name = "holo-configure";
    src = gitignoreSource ./.;
    cargoDir = ".";

    meta.platforms = lib.platforms.all;
  };
}
