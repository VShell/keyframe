{ callPackage, fetchFromGitHub, runCommand, jq, lib, stdenv, nodejs-10_x }:
let
  version = "6.0.1";
  nodePackages = callPackage ./package.nix {
    nodejs = nodejs-10_x;
  };
  converseSource = fetchFromGitHub {
    owner = "conversejs";
    repo = "converse.js";
    rev = "v${version}";
    sha256 = "1al9kh98rzkbf5z55k6f73h0f420ylimnvsmxjq93jlyzjj70g0x";
  };
  patchedConverseSource = runCommand "converse.js-src-patched-${version}" {} ''
    cp -R ${converseSource} $out
    cd $out
    chmod -R u+w .

    # Use a new webpack config
    cp ${./webpack.js} webpack.commonjs.js

    # Merge package.json devDependencies, remove @converse/headless, and remove scripts.prepare
    ${jq}/bin/jq -s '.[0] + {devDependencies: (.[0].devDependencies + .[1].devDependencies)} | del(.devDependencies."@converse/headless") | del(.scripts.prepare)' package.json src/headless/package.json > package.json.new
    mv package.json.new package.json

    # We can't easily write the asset path at runtime, so disable custom emojis for now
    ${jq}/bin/jq '.custom = {}' src/headless/emojis.json > src/headless/emojis.json.new
    mv src/headless/emojis.json{.new,}
  '';
  package = nodePackages.package.override {
    src = runCommand "converse.js-package.json" {} ''
      mkdir $out
      cp ${patchedConverseSource}/package.json $out
      cp ${patchedConverseSource}/package-lock.json $out
    '';
  };
in
stdenv.mkDerivation {
  pname = "converse.js";
  inherit version;
  src = patchedConverseSource;
  configurePhase = ''
    # Add @converse/headless into node_modules
    mkdir node_modules
    find ${package}/lib/node_modules/converse.js/node_modules/ -mindepth 1 -maxdepth 1 \
      -exec ln -s \{\} node_modules \;
    mkdir node_modules/@converse
    ln -s $(pwd)/src/headless node_modules/@converse

    # Rewrite the path so the correct webpack loader runs
    sed -i "s|node_modules/xss|$(readlink node_modules/xss)|" webpack.common.js
  '';
  buildPhase = ''
    ./node_modules/.bin/webpack --config webpack.commonjs.js
  '';
  installPhase = ''
    cp -R dist $out
    cp -R sass/webfonts $out
  '';
}
