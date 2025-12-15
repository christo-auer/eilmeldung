{
  description = "A feature-rich TUI RSS Reader based on the news-flash library";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    {
      overlays.default = final: prev: {
        eilmeldung = final.callPackage ./nix/package.nix {
          inherit (final) llvmPackages_19;
        };
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
          eilmeldung = pkgs.callPackage ./nix/package.nix {
            inherit (pkgs) llvmPackages_19;
          };
          default = self.outputs.packages.${system}.eilmeldung;
        };

        devShells.default = import ./nix/shell.nix { inherit pkgs; };
      }
    );
}
