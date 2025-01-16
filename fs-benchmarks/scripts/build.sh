#!/bin/bash
dfx canister create --all

export WASM_TARGET=wasm32-unknown-unknown
export WASM_TARGET_=`echo $WASM_TARGET | tr '-' '_'`


export RELEASE_DIR=target/$WASM_TARGET/release

rm -f $RELEASE_DIR/*.wasm.gz $RELEASE_DIR/*.wasm 

set -e

pushd `pwd`

if [ "$(basename "$PWD")" = "scripts" ]; then
  cd ..
fi

export PROJECT_NAME="$(basename "$PWD")"
export PROJECT_NAME_=`echo $PROJECT_NAME | tr '-' '_'`

cargo build --release --target $WASM_TARGET

mv $RELEASE_DIR/"$PROJECT_NAME_"_backend.wasm $RELEASE_DIR/built.wasm

ic-wasm $RELEASE_DIR/built.wasm -o $RELEASE_DIR/"$PROJECT_NAME_"_backend.wasm metadata candid:service -f ./src/"$PROJECT_NAME"-backend/"$PROJECT_NAME"-backend.did -v public


popd
