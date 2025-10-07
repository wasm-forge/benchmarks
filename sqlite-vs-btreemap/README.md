# `sqlite-vs-btreemap`

This project compares `SQLite` performance against `StableBTreeMap`.


## Prerequisites

- [rust](https://doc.rust-lang.org/book/ch01-01-installation.html), 
- [dfx](https://internetcomputer.org/docs/current/developer-docs/setup/install/).

Install `wasi2ic` and `canbench`:
```bash
cargo install wasi2ic

cargo install canbench
```

## Run benchmark

To run the benchmark:
```bash
cargo canbench
```

