{ gcc8Stdenv, fetchFromGitHub, pkgconfig, zlib }:

gcc8Stdenv.mkDerivation rec {
  version = "1.0.1";
  pname = "gpac";

  src = fetchFromGitHub {
    owner = "gpac";
    repo = "gpac";
    rev = "v1.0.1";
    sha256 = "0gj46jpprfqv3wyagblv3a52chbplyzhvpra66v63czjibqsslm5";
  };

  patches = [
    ./0001-Check-the-prefix-for-share-and-module-directories.patch
    ./0002-Add-ability-to-inherit-a-UNIX-socket.patch
  ];

  postPatch = ''
    sed -i \
      -e 's/$(IS_DEB_MAKE)/$(origin IS_DEB_MAKE)/' \
      -e 's|/usr/share/|$(prefix)/share/|' \
      Makefile
  '';

  configureFlags = "--enable-debug";
  dontStrip = true;

  nativeBuildInputs = [ pkgconfig zlib ];

  enableParallelBuilding = true;
  foo = "bar";
}
