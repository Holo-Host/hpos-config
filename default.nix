{ pkgs ? import ./pkgs.nix {} }:

with pkgs;

let
  inherit (rust.packages.nightly) rustPlatform;
  inherit (darwin.apple_sdk.frameworks) CoreServices Security;
in

{
  hpos-state-derive = buildRustPackage rustPlatform {
    name = "hpos-state-derive";
    src = gitignoreSource ./.;
    cargoDir = "derive";

    RUST_SODIUM_LIB_DIR = "${libsodium}/lib";
    RUST_SODIUM_SHARED = "1";

    nativeBuildInputs = with buildPackages; [ perl ];
    buildInputs = lib.optionals stdenv.isDarwin [ CoreServices ];

    doCheck = false;
  };

  hpos-state-generate-cli = buildRustPackage rustPlatform {
    name = "hpos-state-generate-cli";
    src = gitignoreSource ./.;
    cargoDir = "generate-cli";

    buildInputs = lib.optionals stdenv.isDarwin [ Security ];

    doCheck = false;
  };

  hpos-state-generate-web = buildRustPackage rustPlatform rec {
    name = "hpos-state-generate-web";
    src = gitignoreSource ./.;
    cargoDir = "generate-web";

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
