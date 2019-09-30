{ pkgs ? import ./pkgs.nix {} }:

with pkgs;

let
  inherit (rust.packages.nightly) rustPlatform;
in

{
  holo-config = buildRustPackage rustPlatform {
    name = "holo-config";
    src = gitignoreSource ./.;

    cargoSha256 = "10jl3wkid0vsy1f6maplmcmkxgjxr75skl79phivfs82ph05ynxs";

    meta.platforms = lib.platforms.all;
  };
}
