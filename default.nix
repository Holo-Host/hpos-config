{ pkgs ? import ./nixpkgs.nix {} }:

with pkgs;

let
  inherit (rust.packages.nightly) rustPlatform;
  inherit (darwin.apple_sdk.frameworks) CoreServices Security;
in

{
  hpos-config-gen-cli = buildRustPackage rustPlatform {
    name = "hpos-config-gen-cli";
    src = gitignoreSource ./.;
    cargoDir = "gen-cli";

    buildInputs = lib.optionals stdenv.isDarwin [ Security ];

    doCheck = false;
  };

  hpos-config-gen-web = buildRustPackage rustPlatform rec {
    name = "hpos-config-gen-web";
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

  hpos-config-into-base36-id = buildRustPackage rustPlatform {
    name = "hpos-config-into-base36-id";
    src = gitignoreSource ./.;
    cargoDir = "into-base36-id";

    buildInputs = lib.optionals stdenv.isDarwin [ Security ];

    doCheck = false;
  };

  hpos-config-into-keystore = buildRustPackage rustPlatform {
    name = "hpos-config-into-keystore";
    src = gitignoreSource ./.;
    cargoDir = "into-keystore";

    RUST_SODIUM_LIB_DIR = "${libsodium}/lib";
    RUST_SODIUM_SHARED = "1";

    nativeBuildInputs = with buildPackages; [ perl ];
    buildInputs = lib.optionals stdenv.isDarwin [ CoreServices ];

    doCheck = false;
  };

  hpos-config-is-valid = buildRustPackage rustPlatform {
    name = "hpos-config-is-valid";
    src = gitignoreSource ./.;
    cargoDir = "is-valid";

    buildInputs = lib.optionals stdenv.isDarwin [ Security ];

    doCheck = false;
  };
}
