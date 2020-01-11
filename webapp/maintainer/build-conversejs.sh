#!/usr/bin/env bash

CONVERSE_VERSION=6.0.0

tmp=$(dirname $(readlink -f $0))/../conversejs
mkdir $tmp
trap "{ rm -rf $tmp; }" EXIT
cd $tmp

git clone https://github.com/conversejs/converse.js.git -b v$CONVERSE_VERSION --depth 1 .

sed -i -e 's|/dist/|/assets/conversejs/|' src/headless/emojis.json

ASSET_PATH=/assets/conversejs/ make build

mv dist/converse{,.min}.{js,css}{,.map} ../vendor/
mv dist/emojis.js{,.map} ../vendor/
rm -rf ../public/assets/conversejs/{locales,webfonts,custom_emojis}
mv dist/{locales,webfonts,custom_emojis} ../public/assets/conversejs/
