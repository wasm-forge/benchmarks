# Run Rusqlite users-orders benchmark

This project shows how to compilte the Rusqlite dependency in order to build the IC canister with the sqlite database.


## Prerequisites

It is assumed that you have [rust](https://doc.rust-lang.org/book/ch01-01-installation.html), 
[dfx](https://internetcomputer.org/docs/current/developer-docs/setup/install/).

You will also need the Wasm-oriented [clang](https://github.com/WebAssembly/wasi-sdk/releases/) installation. 
In this tutorial we use the `.deb` package [installation](https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-24/wasi-sdk-24.0-x86_64-linux.deb). 
Once installed the clang compiler is available from the path `/opt/wasi-sdk/bin/`. The additional builtins library will be found in `/opt/wasi-sdk/lib/clang/*/lib/wasi/`. 


## Preparation

Install wasi2ic and canbench:
```bash
  cargo install wasi2ic
  cargo install canbench
```

## Deployment and testing

To run benchmarks, launch the `canbench` command.

## Pragma settings

This shows the current benchmark results for database performance based on a database file that writes directly to a stable memory (minimized chunked storage overheads).

Following [pragma](https://sqlite.org/pragma.html) settings:

Pragma         | Value                   | Description
---------------|-------------------------|--------------
[journal](https://sqlite.org/pragma.html#pragma_journal_mode)        | PERSIST      | persist journal file (is faster than deleting the file every time). Setting it to `OFF` works faster, but [disallows atomic COMMIT/ROLLBACK](https://sqlite.org/pragma.html#pragma_journal_mode)
[synchronous](https://sqlite.org/pragma.html#synchronous)            | NORMAL       | a readonable value for data safety and performance
[page_size](https://sqlite.org/pragma.html#page_size)                | 4096         | a reasonable default value
[locking_mode](https://sqlite.org/pragma.html#locking_mode)          | EXCLUSIVE    | exclusive mode is faster because we avoid locking and unlocking the database for each query
[temp_store](https://sqlite.org/pragma.html#temp_store)              | MEMORY       | causes to keep the temporary data in memory, at the moment this is necessary to avoid sqlite cash during complex queries
[cache_size](https://sqlite.org/pragma.html#cache_size)              | 1000000      | gives a significant performance boost at the expence of the canister memory used. (It tries to keep the whole database in memory, thus reducing read operation request count)

## Database structure

``` sql
CREATE TABLE users (
  user_id INTEGER PRIMARY KEY AUTOINCREMENT,
  username TEXT NOT NULL,
  email TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE orders (
  order_id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL,
  amount REAL NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (user_id) REFERENCES users(user_id)
);
```

## Benchmark results


Test                  | Cycles cost
----------------------|---------------
Create 100 000 users (cached `INSERT` query with parameters executed 100000 times). 	          | 2.24 B
Create 1M orders (each refers to one of the users, no extra indexes present)                    | 26.04 B
Create indexes, when there are 1M orders in the table: `CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id);`  | 6.24 B
Make a joint selection (100K users, 1M orders): `SELECT u.user_id, u.username, o.order_id, o.amount FROM users u JOIN orders o ON u.user_id = o.user_id WHERE u.user_id < 1000 ORDER BY o.created_at DESC;` | 202.54 M
Select using `LIKE` on an indexed field: `SELECT * FROM users WHERE email LIKE 'user%'`         |	777.95 M
Create 100 extra orders after there were already 1M orders and field indexes created.           |	14.12 M
Remove 1000 orders (we remove all orders from the first 100 users): `DELETE FROM orders WHERE user_id <= 100`                 | 38.40 M
Create 1M orders after indices were created                                                                                   | 36.18 B
Delete 100000 orders with transaction rollback: `BEGIN TRANSACTION; DELETE FROM orders WHERE order_id > 900000; ROLLBACK`     | 2.0 B

