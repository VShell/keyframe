{ callPackage, runCommand, stdenv }:
let
  nodePackages = callPackage ./package.nix {};
  package = nodePackages.package.override {
    # Prune out the source so that we can cache the node_modules tree
    src = runCommand "keyframe-webapp-package.json" {} ''
      mkdir $out
      cp ${./package.json} $out/package.json
      cp ${./package-lock.json} $out/package-lock.json
    '';
  };
in
stdenv.mkDerivation {
  name = "keyframe-webapp";
  src = ./.;
  configurePhase = "";
  buildPhase = ''
    ln -s ${package}/lib/node_modules/keyframe/node_modules node_modules
    export XDG_CONFIG_HOME=$(mktemp -d)
    node_modules/.bin/ember build --environment production --output-path $out/dist/
  '';
  installPhase = ''
    mv $out/dist/index.html $out/index.html
  '';
}
