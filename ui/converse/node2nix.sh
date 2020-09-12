#!/usr/bin/env bash

set -e

script_dir=$(dirname $(readlink -f $0))

tmp=$(mktemp -d)
trap "[[ -d $tmp ]] && rm -r $tmp" EXIT
nix-build -o $tmp/converse -E 'with import <nixpkgs> {}; (callPackage ./. {}).src'

# Strip @converse/headless out of package-lock.json so it doesn't get included in the Nix derivation
jq 'del(.dependencies."@converse/headless")' $tmp/converse/package-lock.json > $tmp/package-lock.json

node2nix -i $tmp/converse/package.json -l $tmp/package-lock.json -o $script_dir/node-packages.nix -c $script_dir/package.nix -e $script_dir/node-env.nix --development
