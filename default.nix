{ pkgs ? import ./pkgs.nix {} }:

with pkgs;

let
  inherit (rust.packages.nightly) rustPlatform;
in

{
  holo-config-derive = buildRustPackage rustPlatform {
    name = "holo-config-derive";
    src = gitignoreSource ./.;
    cargoDir = "derive";

    RUST_SODIUM_LIB_DIR = "${libsodium}/lib";
    RUST_SODIUM_SHARED = "1";

    nativeBuildInputs = with buildPackages; [ perl ];

    doCheck = false;
  };

  holo-config-generate-cli = buildRustPackage rustPlatform {
    name = "holo-config-generate-cli";
    src = gitignoreSource ./.;
    cargoDir = "generate-cli";

    doCheck = false;
  };

  holo-config-generate-web = buildRustPackage rustPlatform rec {
    name = "holo-config-generate-web";
    src = gitignoreSource ./.;
    cargoDir = "generate-web";

    OPENSSL_STATIC = "1";

    nativeBuildInputs = with buildPackages; [
      nodejs-12_x
      pkgconfig
      (wasm-pack.override { inherit rustPlatform; })
    ];

    buildInputs = [ openssl ];

    buildPhase = ''
      cp -r ${npmToNix { src = "${src}/${cargoDir}"; }} node_modules
      chmod -R +w node_modules
      chmod +x node_modules/.bin/webpack
      patchShebangs node_modules

      npm run build
    '';

    installPhase = ''
      mv target/webpack $out
    '';

    doCheck = false;
  };
}
