#!/bin/bash


dfx canister create --all

set -e


rm -rf target/wasm32-wasip1/*.wasm

cargo build --release --target wasm32-wasip1

wasi2ic target/wasm32-wasip1/release/sql_users_orders_backend.wasm target/wasm32-wasip1/release/no_wasi.wasm

#gzip -f target/wasm32-wasip1/release/no_wasi.wasm > target/wasm32-wasip1/release/no_wasi.wasm.gz

