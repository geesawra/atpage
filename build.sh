#!/bin/bash
set -e
set -x
rm -rf pkg
wasm-pack build --release --no-typescript --target web atresolver
mkdir pkg
cp atresolver/pkg/* pkg
