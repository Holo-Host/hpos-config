{ pkgs ? import ./pkgs.nix {} }:

with pkgs;

let
  inherit (rust.packages.nightly) rustPlatform;
  inherit (darwin.apple_sdk.frameworks) CoreServices Security;
in

{
  hpos-state-derive-keystore = buildRustPackage rustPlatform {
    name = "hpos-state-derive-keystore";
    src = gitignoreSource ./.;
    cargoDir = "derive-keystore";

    RUST_SODIUM_LIB_DIR = "${libsodium}/lib";
    RUST_SODIUM_SHARED = "1";

    nativeBuildInputs = with buildPackages; [ perl ];
    buildInputs = lib.optionals stdenv.isDarwin [ CoreServices ];

    doCheck = false;
  };

  hpos-state-gen-cli = buildRustPackage rustPlatform {
    name = "hpos-state-gen-cli";
    src = gitignoreSource ./.;
    cargoDir = "gen-cli";

    buildInputs = lib.optionals stdenv.isDarwin [ Security ];

    doCheck = false;
  };

  hpos-state-gen-web = buildRustPackage rustPlatform rec {
    name = "hpos-state-gen-web";
    src = gitignoreSource ./.;
    cargoDir = "gen-web";

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
      mv dist $out
    '';

    doCheck = false;
  };
}
