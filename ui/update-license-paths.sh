#!/usr/bin/env bash

set -e

script_dir=$(dirname $(readlink -f $0))

tmp=$(mktemp -d)
trap "[[ -d $tmp ]] && rm -r $tmp" EXIT

(
echo '{'
jq -r 'to_entries[]|.key+"\t"+.value.licenseFile' $script_dir/src/backend-licenses.json | while IFS=$'\t' read -r key file; do
  printf '%s = ./../%s;\n' $key $file
done
echo '}'
) > licenses.nix
