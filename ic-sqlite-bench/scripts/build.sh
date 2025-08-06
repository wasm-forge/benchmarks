#!/bin/bash

dfx canister create --all

export RELEASE_DIR=target/wasm32-wasip1/release

rm -f $RELEASE_DIR/*.wasm.gz $RELEASE_DIR/*.wasm 

set -e

cargo build --release --target wasm32-wasip1

wasi2ic $RELEASE_DIR/ic_sqlite_bench_backend.wasm $RELEASE_DIR/no_wasi.wasm

gzip -f $RELEASE_DIR/no_wasi.wasm > $RELEASE_DIR/no_wasi.wasm.gz
