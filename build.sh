#!/bin/bash
set -e
set -x
rm -rf pkg
wasm-pack build --release --no-typescript --target web
mkdir -p public/pkg
cp pkg/* public/pkg
cp index.html public
cp sw.js public
