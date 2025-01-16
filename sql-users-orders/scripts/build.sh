#!/bin/bash
dfx canister create --all

export WASI_TARGET=wasm32-wasip1
export WASI_TARGET_=`echo $WASI_TARGET | tr '-' '_'`

export CC_$WASI_TARGET_="/opt/wasi-sdk/bin/clang"
export CFLAGS_$WASI_TARGET_="--sysroot=/opt/wasi-sdk/share/wasi-sysroot"

export RELEASE_DIR=target/$WASI_TARGET/release

rm -f $RELEASE_DIR/*.wasm.gz $RELEASE_DIR/*.wasm 

set -e

pushd `pwd`

if [ "$(basename "$PWD")" = "scripts" ]; then
  cd ..
fi

export PROJECT_NAME="$(basename "$PWD")"
export PROJECT_NAME_=`echo $PROJECT_NAME | tr '-' '_'`

cargo build --release --target $WASI_TARGET

mv $RELEASE_DIR/"$PROJECT_NAME_"_backend.wasm $RELEASE_DIR/built.wasm

ic-wasm $RELEASE_DIR/built.wasm -o $RELEASE_DIR/meta.wasm metadata candid:service -f ./src/"$PROJECT_NAME"-backend/"$PROJECT_NAME"-backend.did -v public

wasi2ic $RELEASE_DIR/meta.wasm $RELEASE_DIR/no_wasi.wasm

cp $RELEASE_DIR/no_wasi.wasm $RELEASE_DIR/"$PROJECT_NAME_"_backend.wasm

gzip -f $RELEASE_DIR/no_wasi.wasm > $RELEASE_DIR/no_wasi.wasm.gz

popd
