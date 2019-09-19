{ pkgs ? import ./pkgs.nix {} }:

with pkgs;

{
  holo-configure = rustPlatform.buildRustPackage {
    name = "holo-configure";
    src = gitignoreSource ./.;

    cargoSha256 = "10g8w0pvvxhdr3bax3ly5f619b3mn2j810rbmcbgibg077198b7h";

    meta.platforms = lib.platforms.all;
  };
}
