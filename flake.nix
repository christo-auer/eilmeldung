{
  description = "Rust Development Shell";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs.buildPackages;
      {
        devShells.default = mkShell {
          nativeBuildInputs = [
            pkg-config
            cmake
            cargo
            rust-analyzer
            rustc
            clippy
            rustfmt
            llvmPackages_19.libclang
            llvmPackages_19.clang
          ];

          buildInputs = [
            openssl
            libxml2
            sqlite
            glibc
            glibc.dev
          ];

          LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_19.libclang.lib ];
          BINDGEN_EXTRA_CLANG_ARGS = 
            (builtins.map (a: ''-I"${a}/include"'') [
# add dev libraries here (e.g. pkgs.libvmi.dev)
             pkgs.glibc.dev
            ])
# Includes with special directory paths
            ++ [
              ''-I"${pkgs.llvmPackages.libclang.lib}/lib/clang/19/include"''
              ''-I"${pkgs.glib.dev}/include/glib-2.0"''
              ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
              ''-I"${pkgs.glibc.dev}/include/"''
            ];        
        };
      }
    );
}

