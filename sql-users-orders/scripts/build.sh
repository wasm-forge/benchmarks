#!/bin/bash
dfx canister create --all

export WASI_TARGET=wasm32-wasi
export WASI_TARGET_=wasm32_wasi

export CC_$WASI_TARGET_="/opt/wasi-sdk/bin/clang"
export CFLAGS_$WASI_TARGET_="--sysroot=/opt/wasi-sdk/share/wasi-sysroot"

export RELEASE_DIR=target/$WASI_TARGET/release

rm -f $RELEASE_DIR/no_wasi.wasm.gz $RELEASE_DIR/built.wasm $RELEASE_DIR/no_wasi.wasm $RELEASE_DIR/meta.wasm $RELEASE_DIR/meta.wasm.gz 

set -e

pushd `pwd`

if [ "$(basename "$PWD")" = "scripts" ]; then
  cd ..
fi

cargo build --release --target $WASI_TARGET

mv $RELEASE_DIR/sql_users_orders_backend.wasm $RELEASE_DIR/built.wasm

ic-wasm $RELEASE_DIR/built.wasm -o $RELEASE_DIR/meta.wasm metadata candid:service -f ./src/sql-users-orders-backend/sql-users-orders-backend.did -v public

wasi2ic $RELEASE_DIR/meta.wasm $RELEASE_DIR/no_wasi.wasm

cp $RELEASE_DIR/no_wasi.wasm $RELEASE_DIR/sql_users_orders_backend.wasm

gzip -f $RELEASE_DIR/no_wasi.wasm > $RELEASE_DIR/no_wasi.wasm.gz

popd
