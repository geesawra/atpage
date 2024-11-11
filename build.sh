#!/bin/bash
set -e
set -x
rm -rf pkg
wasm-pack build --release --no-typescript --target web atresolver
# wasm-pack build --release --no-typescript --target web atproto
mkdir pkg
cp atproto/pkg/* pkg
cp atresolver/pkg/* pkg
