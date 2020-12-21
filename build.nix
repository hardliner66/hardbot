# import niv sources and the pinned nixpkgs
{ sources ? import ./nix/sources.nix, pkgs ? import sources.nixpkgs { }}:
let
  
  # configure naersk to use our pinned rust compiler
  naersk = pkgs.callPackage sources.naersk {
    rustc = pkgs.rustc;
    cargo = pkgs.cargo;
  };
  
  # tell nix-build to ignore the `target` directory
  src = builtins.filterSource
    (path: type: type != "directory" || builtins.baseNameOf path != "target")
    ./.;
in naersk.buildPackage {
  inherit src;
  remapPathPrefix =
    true; # remove nix store references for a smaller output package
}
