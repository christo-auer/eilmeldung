{
  description = "A feature-rich TUI RSS Reader based on the news-flash library";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    let
      version = "1.7.3";

      releaseSrc = pkgs: pkgs.fetchFromGitHub {
        owner = "christo-auer";
        repo = "eilmeldung";
        rev = version;
        hash = "sha256-QCGtuf1XSLpWr72GYUsz20JllWHNJ7Q4cAtNTywi4JM=";
      };

      mkEilmeldung = pkgs: src: ver:
        (pkgs.callPackage ./nix/package.nix {
          inherit (pkgs) llvmPackages_19;
        }) { inherit src; version = ver; };

      deprecationWarning = ''
      eilmeldung has moved into nixpkgs unstable (pkgs.eilmeldung) and the
      home-manager module has moved into the official home-manager repository
      (master branch)! Installing and configuring eilmeldung via this flake is
      **deprecated**. This flake will receive updates until the next release of
      nixos and home-manager. After that, this warning will be an error.
      '';
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
          eilmeldung = pkgs.lib.warn deprecationWarning (mkEilmeldung pkgs (releaseSrc pkgs) version);
          eilmeldung-git = pkgs.lib.warn deprecationWarning (mkEilmeldung pkgs self (self.shortRev or "dirty"));
          default = pkgs.lib.warn deprecationWarning (self.outputs.packages.${system}.eilmeldung);
        };

        devShells.default = import ./nix/shell.nix { inherit pkgs; };
      }
    );
}
