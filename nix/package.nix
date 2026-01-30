{ lib
, rustPlatform
, pkg-config
, cmake
, perl
, openssl
, libxml2
, sqlite
, glib
, glibc
, llvmPackages_19
}:

rustPlatform.buildRustPackage {
  pname = "eilmeldung";
  version = "0.8.1";
  
  src = ../.;
  
  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes = {
      "news-flash-3.0.0-alpha.0" = "sha256-vrlQsy3e1URSO3ZzP17/qHtFhkPpESbwxpPD3mmMbps=";
      "tui-textarea-0.7.0" = "sha256-3ENi0XCVkhJAj9mgMXXkCY2FZ1VcVrSjfidBCsYdfMA=";
    };
  };
  
  nativeBuildInputs = [
    pkg-config
    cmake
    perl  
  ];
  
  buildInputs = [
    openssl
    libxml2
    sqlite
  ];
  
  LIBCLANG_PATH = lib.makeLibraryPath [ llvmPackages_19.libclang.lib ];
  BINDGEN_EXTRA_CLANG_ARGS = lib.concatStringsSep " " (
    (builtins.map (a: ''-I"${a}/include"'') [
      glibc.dev
    ])
    ++ [
      ''-I"${llvmPackages_19.libclang.lib}/lib/clang/19/include"''
      ''-I"${glib.dev}/include/glib-2.0"''
      ''-I${glib.out}/lib/glib-2.0/include/''
      ''-I"${glibc.dev}/include/"''
    ]
  );
  
  meta = with lib; {
    description = "A feature-rich TUI RSS Reader based on the news-flash library";
    homepage = "https://github.com/christo-auer/eilmeldung";
    license = licenses.gpl3Plus;
    maintainers = [ "christo-auer" ];
    mainProgram = "eilmeldung";
  };
}
