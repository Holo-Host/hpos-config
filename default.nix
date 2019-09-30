{ pkgs ? import ./pkgs.nix {} }:

with pkgs;

let
  inherit (rust.packages.nightly) rustPlatform;
in

{
  holo-config = buildRustPackage rustPlatform {
    name = "holo-config";
    src = gitignoreSource ./.;

    cargoSha256 = "0000000000000000000000000000000000000000000000000000";

    meta.platforms = lib.platforms.all;
  };
}
