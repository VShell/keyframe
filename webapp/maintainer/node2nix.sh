#!/usr/bin/env bash

cd $(dirname $(readlink -f $0))/..
node2nix -l package-lock.json --composition package.nix --development
