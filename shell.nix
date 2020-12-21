let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {};
in
pkgs.mkShell {
  buildInputs = with pkgs; [ openssl pkg-config rustc cargo rustfmt rustPackages.clippy cargo-edit ];
}
