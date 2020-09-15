{ stdenv, lib, callPackage, jq }:
let
  converse = callPackage ./converse {};
  nodePackages = callPackage ./package.nix {};
  package = nodePackages.package.override {
    src = lib.sourceByRegex ./. [ "^package\\.json$" "^package-lock\\.json$" ];
  };
  licenses = import ./licenses.nix;
in
stdenv.mkDerivation {
  name = "keyframe-ui";
  src = ./.;
  passthru = {
    node_modules = "${package}/lib/node_modules/keyframe-ui/node_modules";
  };
  postPatch = ''
    ${lib.concatStrings (lib.mapAttrsToList (key: licenseFile:
      ''
        ${jq}/bin/jq --arg key ${key} --arg licenseFile ${licenseFile} '. * {($key): {"licenseFile": $licenseFile}}' src/backend-licenses.json > src/backend-licenses.json.new
        mv src/backend-licenses.json{.new,}
      ''
    ) licenses)}
  '';
  configurePhase = ''
    ln -s ${package}/lib/node_modules/keyframe-ui/node_modules node_modules
    ln -s ${converse} converse/dist
  '';
  buildPhase = ''
    ./node_modules/.bin/vue-cli-service build --dest $out
  '';
  installPhase = "#";
}
