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
  version = "0.7.7";
  
  src = ../.;
  
  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes = {
      "nanohtml2text-0.2.1" = "sha256-HyucvnpG6H9NOG1UdIP/X1za03sA3xuLxPG8FW3zsWo=";
      "news-flash-2.3.0-alpha.0" = "sha256-vU3IlBA4c+i77Ux/rUmiasxOlcQSYC8c1tbSoRzUcjY=";
      "newsblur_api-0.3.1" = "sha256-/q4I5ZywnvGPDxwH1bCxk1+AmN0t2MRsCRMsOjmfRzI=";
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
  BINDGEN_EXTRA_CLANG_ARGS = builtins.concatStringsSep " " (
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
