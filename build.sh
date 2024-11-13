#!/bin/bash
set -e
set -x
rm -rf pkg
wasm-pack build --release --no-typescript --target web atresolver
mkdir -p public/pkg
cp atresolver/pkg/* public/pkg
cp index.html public
