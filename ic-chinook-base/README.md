# `ic-rusqlite` Chinook Database Benchmark

This project benchmarks `ic-rusqlite` dependency to test work with a large database.

## Prerequisites

- [rust](https://doc.rust-lang.org/book/ch01-01-installation.html), 
- [dfx](https://internetcomputer.org/docs/current/developer-docs/setup/install/).

### Setting Up the Environment

You can setup your build environment via script:
```sh
curl -fsSL https://raw.githubusercontent.com/wasm-forge/ic-rusqlite/main/prepare.sh | sh
```

The script will:
- install `wasi2ic`: `cargo install wasi2ic`
- install WASI target: `rustup target add wasm32-wasip1`
- download `WASI-SDK` and WASI-oriented `clang`: [WASI-SDK](https://github.com/WebAssembly/wasi-sdk/releases/). 
- after confirmation, it will define necessary variables in your `.bashrc`:
```sh
export WASI_SDK=<path to wasi-sdk>
export PATH=$WASI_SDK/bin:$PATH
```
## Chinook Database

![Chinook Entity Relationship Diagram](img/chinook-erd.png)

## Deploy canister

Start DFX:
```sh
dfx start --clean --background
```

Deploy thoe canister:
```sh
dfx deploy
```

## Download database

Download the Chinook database from the [SQLite tutorial](https://www.sqlitetutorial.net/) site:
```sh
./scripts/sample_download.sh
```


## Database structure

