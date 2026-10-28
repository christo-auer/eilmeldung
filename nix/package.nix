{ src, version }:

{ lib
, rustPlatform
}:

rustPlatform.buildRustPackage {
  pname = "eilmeldung";
  inherit version src;

  cargoLock = {
    lockFile = "${src}/Cargo.lock";
  };

  meta = {
    description = "Feature-rich TUI RSS Reader based on the news-flash library";
    homepage = "https://github.com/christo-auer/eilmeldung";
    license = lib.licenses.gpl3Plus;
    #maintainers = with lib.maintainers [ christo-auer ];
    mainProgram = "eilmeldung";
  };
}
