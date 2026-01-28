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
      "nanohtml2text-0.2.1" = "sha256-HyucvnpG6H9NOG1UdIP/X1za03sA3xuLxPG8FW3zsWo=";
      "news-flash-2.3.0-alpha.0" = "sha256-+o/O+GYktfHnCqCvRE3aguT9w2thNviMkCYSCtvuwJU=";
      "newsblur_api-0.4.0" = "sha256-3FcfCFxX74uxMkTieGlDH9T+5snlH0j7+0vpswzgdVE=";
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
