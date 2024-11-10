# Run Rusqlite users-orders benchmark

This project shows how to compilte the Rusqlite dependency in order to build the IC canister with the sqlite database.


## Prerequisites

It is assumed that you have [rust](https://doc.rust-lang.org/book/ch01-01-installation.html), 
[dfx](https://internetcomputer.org/docs/current/developer-docs/setup/install/).



You will also need the Wasm-oriented [clang](https://github.com/WebAssembly/wasi-sdk/releases/) installation. In this tutorial we use the `.deb` package [installation](https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-24/wasi-sdk-24.0-x86_64-linux.deb). Once installed the clang
compiler is available from the path `/opt/wasi-sdk/bin/`. The additional builtins library will be found in `/opt/wasi-sdk/lib/clang/18/lib/wasi/`. 


## Preparation

Install wasi2ic and canbench:
```bash
  cargo install wasi2ic
  cargo install canbench
```

## Deployment and testing

To run benchmarks, launch the `canbench` command.
