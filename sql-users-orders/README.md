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

## Benchmark settings

This shows the current benchmark results for database performance based on mounted memory database file, and the usual "journal" file setup.

Following [pragma](https://sqlite.org/pragma.html) settings:

Pragma         | Value                   | Description
---------------|-------------------------|--------------
journal        | TRUNCATE                | this option works slower than OFF, but it allows [rollbacks](https://sqlite.org/pragma.html#pragma_journal_mode)
synchronous    | NORMAL                  | a readonable value for data safety and performance
page_size      | 4096                    | a reasonable default value
locking_mode   | EXCLUSIVE               | exclusive mode is faster because we avoid locking and unlocking the database for each query
temp_store     | MEMORY                  | causes to keep the temporary data in memory, at the moment this is necessary to avoid sqlite cash during complex queries
cache_size     | 1000000                 | gives a significant performance boost at the expence of the canister memory used. (It tries to keep the whole database in memory, thus reducing read operation request count)

## Benchmark results


Test                  | Cycles cost
----------------------|---------------
Create 100 000 users 	                                                                          | 2.27 B
Create 1M orders (each refers to one of the users, no extra indexes present)                    | 25.99 B
Create indexes on fields when the orders is filled with data: users.email and orders.user_id    | 6.24 B
Make a joint selection: `SELECT u.user_id, u.username, o.order_id, o.amount FROM users u JOIN orders o ON u.user_id = o.user_id WHERE u.user_id < 1000 ORDER BY o.created_at DESC;` | 202.12 M
Select using "like" on an indexed field: `SELECT * FROM users WHERE email LIKE 'user%'`         |	778.70 M
Create 100 extra orders after there were already 1M orders and field indexes created.           |	15.68 M
Remove 1000 orders (each user has 10 orders, we remove from the first 100 users): `DELETE FROM orders WHERE user_id <= 100` | 90.10 M
Delete 100000 orders with transaction rollback: `BEGIN TRANSACTION DELETE FROM orders WHERE order_id > 900000 ROLLBACK`     | 2.25B

