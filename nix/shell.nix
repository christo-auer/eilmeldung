{ pkgs }:

pkgs.mkShell {
  # Development tools
  nativeBuildInputs = with pkgs; [
    # Rust toolchain
    cargo
    rustc
    rust-analyzer
    clippy
    rustfmt
    rustPlatform.bindgenHook

    # Build tools
    pkg-config
    cmake
    perl
  ];

  # Libraries needed for building
  buildInputs = with pkgs; [
    openssl
    libxml2
    sqlite
  ];
}
