#!/bin/bash
dfx canister create --all

export RELEASE_DIR=target/wasm32_unknown_unknown/release

# adds about 10% improvement to some operations
#export RUSTFLAGS='-C target-feature=+bulk-memory'

set -e

cargo build --release --target wasm32-unknown-unknown
