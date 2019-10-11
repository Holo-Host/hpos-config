{ pkgs ? import ./pkgs.nix {} }:

with pkgs;

let
  inherit (rust.packages.nightly) rustPlatform;

  wasm-pack = pkgs.wasm-pack.override {
    inherit rustPlatform;
  };
in

{
  holo-config = buildRustPackage rustPlatform {
    name = "holo-config";
    src = gitignoreSource ./.;

    cargoSha256 = "10jl3wkid0vsy1f6maplmcmkxgjxr75skl79phivfs82ph05ynxs";

    nativeBuildInputs = with buildPackages; [
      nodejs-12_x
      openssl.dev
      perl
      wasm-pack
    ];

    OPENSSL_STATIC = "1";
    RUST_SODIUM_LIB_DIR = "${libsodium}/lib";
    RUST_SODIUM_SHARED = "1";

    meta.platforms = lib.platforms.all;
  };
}
