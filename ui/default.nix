{ stdenv, lib, callPackage }:
let
  converse = callPackage ./converse {};
  nodePackages = callPackage ./package.nix {};
  package = nodePackages.package.override {
    src = lib.sourceByRegex ./. [ "^package\\.json$" "^package-lock\\.json$" ];
  };
in
stdenv.mkDerivation {
  name = "keyframe-ui";
  src = ./.;
  passthru = {
    node_modules = "${package}/lib/node_modules/keyframe-ui/node_modules";
  };
  configurePhase = ''
    ln -s ${package}/lib/node_modules/keyframe-ui/node_modules node_modules
    ln -s ${converse} converse/dist
  '';
  buildPhase = ''
    ./node_modules/.bin/vue-cli-service build --dest $out
  '';
  installPhase = "#";
}
