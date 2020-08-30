{ pkgs ? import <nixpkgs> {} }:
with pkgs;
mkShell {
    buildInputs = [ (rustChannelOf { date = "2020-06-06"; }).rust pkgconfig srt clang openssl ];
}
