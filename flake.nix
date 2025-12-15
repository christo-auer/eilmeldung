{
  description = "A feature-rich TUI RSS Reader based on the news-flash library";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    {
      # Overlay - adds eilmeldung to pkgs
      overlays.default = final: prev: {
        eilmeldung = final.callPackage ./nix/package.nix {
          inherit (final) llvmPackages_19;
        };
      };
      
      # Home Manager module - exposed at the top level (not per-system)
      homeManagerModules.default = import ./nix/home-manager-module.nix;
      homeManagerModules.eilmeldung = self.outputs.homeManagerModules.default;
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
