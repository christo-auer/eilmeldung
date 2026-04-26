{
  description = "A feature-rich TUI RSS Reader based on the news-flash library";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    let
      version = "1.4.4";

      releaseSrc = pkgs: pkgs.fetchFromGitHub {
        owner = "christo-auer";
        repo = "eilmeldung";
        rev = version;
        hash = "sha256-s3lNNOKD+fJJMVJsFarNSoljewHqxovUpmz+VVbyfLo=";
      };

      mkEilmeldung = pkgs: src: ver:
        (pkgs.callPackage ./nix/package.nix {
          inherit (pkgs) llvmPackages_19;
        }) { inherit src; version = ver; };
    in
    {
      overlays.default = final: prev: {
        eilmeldung = mkEilmeldung final (releaseSrc final) version;
      };

      homeManager.default = import ./nix/home-manager-module.nix;
      homeManager.eilmeldung = self.outputs.homeManager.default;
    }
    // flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        packages = {
          eilmeldung = mkEilmeldung pkgs (releaseSrc pkgs) version;
          eilmeldung-git = mkEilmeldung pkgs self (self.shortRev or "dirty");
          default = self.outputs.packages.${system}.eilmeldung;
        };

        devShells.default = import ./nix/shell.nix { inherit pkgs; };
      }
    );
}
