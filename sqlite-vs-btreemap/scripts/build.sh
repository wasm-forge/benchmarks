#!/bin/bash


dfx canister create --all

set -e


rm -rf target/wasm32-wasip1/*.wasm

cargo build --release --target wasm32-wasip1

wasi2ic target/wasm32-wasip1/release/sqlite_vs_btreemap_backend.wasm target/wasm32-wasip1/release/no_wasi.wasm



