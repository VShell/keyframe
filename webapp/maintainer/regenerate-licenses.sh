#!/usr/bin/env bash

dir=$(dirname $(readlink -f $0))
$dir/../node_modules/.bin/license-checker --json --customPath $dir/license-custom-format.json --out $dir/../public/assets/licenses.json
