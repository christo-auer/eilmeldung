# Development shell for eilmeldung
# This provides all tools needed for development (not just building)
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
    
    # Build tools
    pkg-config
    cmake
    
    # Debugging and profiling
    llvmPackages_19.libclang
    llvmPackages_19.clang
    gdb
    valgrind
  ];

  # Libraries needed for building
  buildInputs = with pkgs; [
    openssl
    libxml2
    sqlite
    glibc
    glibc.dev
  ];

  # Environment variables for development
  LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_19.libclang.lib ];
  BINDGEN_EXTRA_CLANG_ARGS = builtins.concatStringsSep " " (
    (builtins.map (a: ''-I"${a}/include"'') [
      pkgs.glibc.dev
    ])
    ++ [
      ''-I"${pkgs.llvmPackages_19.libclang.lib}/lib/clang/19/include"''
      ''-I"${pkgs.glib.dev}/include/glib-2.0"''
      ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
      ''-I"${pkgs.glibc.dev}/include/"''
    ]
  );
  
  # Shell hook - runs when entering the dev shell
  shellHook = ''
    echo "🦀 Rust development environment for eilmeldung"
    echo "Run 'cargo build' to build, 'cargo run' to run"
  '';
}
