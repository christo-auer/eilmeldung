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
, stdenv
, llvmPackages_19
}:

{ src, version }:

rustPlatform.buildRustPackage {
  pname = "eilmeldung";
  inherit version src;

  cargoLock = {
    lockFile = "${src}/Cargo.lock";
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
    lib.optionals stdenv.isLinux [
      ''-I"${glibc.dev}/include"''
      ''-I"${glibc.dev}/include/"''
    ]
    ++ [
      ''-I"${llvmPackages_19.libclang.lib}/lib/clang/19/include"''
      ''-I"${glib.dev}/include/glib-2.0"''
      ''-I${glib.out}/lib/glib-2.0/include/''
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
