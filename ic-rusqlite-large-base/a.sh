#!/bin/bash

rm target/wasm32-wasip1/release/no_wasi.wasm.gz
rm target/wasm32-wasip1/release/*.wasm

dfx deploy

./scripts/test.sh


