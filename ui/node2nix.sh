#!/usr/bin/env bash

set -e

script_dir=$(dirname $(readlink -f $0))

node2nix -i $script_dir/package.json -l $script_dir/package-lock.json -o $script_dir/node-packages.nix -c $script_dir/package.nix -e $script_dir/node-env.nix --development
