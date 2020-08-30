{ runCommandCC, pkgconfig, systemd }:

runCommandCC "systemd-socket-activation-shim" {
  nativeBuildInputs = [ pkgconfig ];
  buildInputs = [ systemd ];
} ''
mkdir -p $out/bin
$CC $(pkg-config --cflags --libs libsystemd) ${./systemd-socket-activation-shim.c} -o $out/bin/systemd-socket-activation-shim
''
