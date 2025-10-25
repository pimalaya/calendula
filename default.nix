{
  pimalaya ? import (fetchTarball "https://github.com/pimalaya/nix/archive/master.tar.gz"),
  ...
}@args:

let
  package = {
    src = ./.;
    version = "0.1.0";
    mkPackage = (
      {
        lib,
        pkgs,
        rustPlatform,
        defaultFeatures,
        features,
      }:

      pkgs.callPackage ./package.nix {
        inherit lib rustPlatform;
        apple-sdk = pkgs.apple-sdk;
        installShellCompletions = false;
        installManPages = false;
        withNoDefaultFeatures = !defaultFeatures;
        withFeatures = lib.splitString "," features;
      }
    );
  };
in

pimalaya.mkDefault (package // removeAttrs args [ "pimalaya" ])
