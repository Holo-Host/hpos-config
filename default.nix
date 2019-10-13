{ pkgs ? import ./pkgs.nix {} }:

with pkgs;

let
  inherit (rust.packages.nightly) rustPlatform;
in

{
  holo-config-derive = buildRustPackage rustPlatform {
    name = "holo-config-derive";
    src = gitignoreSource ./derive;

    cargoSha256 = "0000000000000000000000000000000000000000000000000000";

    nativeBuildInputs = with buildPackages; [ perl ];

    OPENSSL_STATIC = "1";
    RUST_SODIUM_LIB_DIR = "${libsodium}/lib";
    RUST_SODIUM_SHARED = "1";
  };

  holo-config-generate-cli = buildRustPackage rustPlatform {
    name = "holo-config-generate-cli";
    src = gitignoreSource ./generate-cli;
    cargoDir = ".";
  };

  holo-config-generate-web = buildRustPackage rustPlatform {
    name = "holo-config-generate-web";
    src = gitignoreSource ./generate-web;
    cargoDir = ".";

    nativeBuildInputs = [
      nodejs-12_x
      (wasm-pack.override { inherit rustPlatform; })
    ];
  };
}
