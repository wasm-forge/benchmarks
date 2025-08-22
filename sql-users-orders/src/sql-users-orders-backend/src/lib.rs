use candid::CandidType;
use candid::Deserialize;

use ic_rusqlite::types::Type;

use ic_rusqlite::with_connection;

type Result<T = String, E = Error> = std::result::Result<T, E>;

type QueryResult<T = Vec<Vec<Option<String>>>, E = Error> = std::result::Result<T, E>;

#[ic_cdk::query]
fn query(sql: String) -> QueryResult {
    with_connection(|conn| {
        let mut stmt = conn.prepare(&sql).unwrap();
        let cnt = stmt.column_count();

        let mut rows = stmt.query([]).unwrap();

        let mut res: Vec<Vec<Option<String>>> = Vec::new();

        loop {
            match rows.next() {
                Ok(row) => match row {
                    Some(row) => {
                        let mut vec: Vec<Option<String>> = Vec::new();
                        for idx in 0..cnt {
                            let v = row.get_ref_unwrap(idx);
                            match v.data_type() {
                                Type::Null => vec.push(None),
                                Type::Integer => vec.push(Some(v.as_i64().unwrap().to_string())),
                                Type::Real => vec.push(Some(v.as_f64().unwrap().to_string())),
                                Type::Text => vec.push(Some(v.as_str().unwrap().parse().unwrap())),
                                Type::Blob => vec.push(Some(hex::encode(v.as_blob().unwrap()))),
                            }
                        }
                        res.push(vec)
                    }
                    None => break,
                },
                Err(err) => {
                    return Err(Error::CanisterError {
                        message: format!("{err:?}"),
                    })
                }
            }
        }
        Ok(res)
    })
}

fn execute(sql: &str) {
    with_connection(|conn| {
        conn.execute(sql, ()).unwrap();
    })
}

#[ic_cdk::update]
fn create_tables() {
    execute(
        "
        CREATE TABLE IF NOT EXISTS users (
                        user_id INTEGER PRIMARY KEY AUTOINCREMENT,
                        username TEXT NOT NULL,
                        email TEXT NOT NULL,
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )
       ",
    );

    execute(
        "
            CREATE TABLE IF NOT EXISTS orders (
                order_id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                amount REAL NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES users(user_id)
            )
        ",
    );
}

#[ic_cdk::update]
fn create_indices() {
    execute("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);");
    execute("CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id);");
}

#[ic_cdk::init]
fn init() {
    create_tables();
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    init();
}

#[derive(CandidType, Deserialize, Debug)]
enum Error {
    InvalidCanister,
    CanisterError { message: String },
}

#[ic_cdk::update]
fn add_users(offset: u64, count: u64) -> Result {
    with_connection(|mut conn| {
        let tx = conn.transaction().unwrap();

        let sql = String::from("insert into users (username, email) values (?, ?)");

        {
            let mut stmt = tx.prepare_cached(&sql).unwrap();

            let mut i = 0;

            while i < count {
                let id = offset + i + 1;
                let username = format!("user{id}");
                let email = format!("user{id}@example.com");

                stmt.execute(ic_rusqlite::params![username, email])
                    .expect("insert of a user failed!");

                i += 1;
            }
        }

        tx.commit().expect("COMMIT USER INSERTION FAILED!");

        Ok(String::from("bench1_insert_person OK"))
    })
}

#[ic_cdk::update]
fn add_orders(offset: u64, count: u64, id_mod: u64) -> Result {
    with_connection(|mut conn| {
        let tx = conn.transaction().unwrap();

        let sql = String::from("insert into orders (user_id, amount) values (?, ?)");

        {
            let mut stmt = tx.prepare_cached(&sql).unwrap();

            let mut i = 0;

            while i < count {
                let id = (offset + i + 1) * 13 % id_mod + 1;

                stmt.execute(ic_rusqlite::params![id, (id * 100 + id * 17) / 15])
                    .unwrap_or_else(|_| {
                        panic!(
                            "insertion of a new order failed: i = {i} count = {count} id = {id}!"
                        )
                    });

                i += 1;
            }
        }

        tx.commit().expect("COMMIT ORDER INSERTION FAILED!");

        Ok("add_orders OK".to_string())
    })
}

mod benches {
    use super::*;
    use canbench_rs::{bench, bench_fn, BenchResult};

    const COUNT: u64 = 1000000u64;

    #[bench(raw)]
    fn bench_add_users() -> BenchResult {
        bench_fn(|| {
            add_users(0, COUNT / 10).unwrap();
        })
    }

    #[bench(raw)]
    fn bench_add_orders() -> BenchResult {
        add_users(0, COUNT / 10).unwrap();

        bench_fn(|| {
            add_orders(0, COUNT, COUNT / 10).unwrap();
        })
    }

    #[bench(raw)]
    fn bench_add_indices() -> BenchResult {
        add_users(0, COUNT / 10).unwrap();
        add_orders(0, COUNT, COUNT / 10).unwrap();

        bench_fn(|| {
            create_indices();
        })
    }

    #[bench(raw)]
    fn bench_select_with_join() -> BenchResult {
        add_users(0, COUNT / 10).unwrap();
        add_orders(0, COUNT, COUNT / 10).unwrap();
        create_indices();

        bench_fn(|| {
            query(
                r#"
                SELECT u.user_id, u.username, o.order_id, o.amount
                FROM users u
                JOIN orders o ON u.user_id = o.user_id
                WHERE u.user_id < 1000
                ORDER BY o.created_at DESC;
                "#
                .to_string(),
            )
            .unwrap();
        })
    }

    #[bench(raw)]
    fn bench_select_like_on_indexed_field() -> BenchResult {
        let user_count = COUNT / 10;
        add_users(0, user_count).unwrap();
        add_orders(0, COUNT, user_count).unwrap();
        create_indices();

        bench_fn(|| {
            query(
                r#"
                SELECT * FROM users WHERE email LIKE 'user%';
                "#
                .to_string(),
            )
            .unwrap();
        })
    }

    #[bench(raw)]
    fn bench_add_100_indexed_orders() -> BenchResult {
        let user_count = COUNT / 10;
        add_users(0, user_count).unwrap();
        add_orders(0, COUNT, user_count).unwrap();
        create_indices();

        bench_fn(|| {
            add_orders(0, 100, user_count).unwrap();
        })
    }

    #[bench(raw)]
    fn bench_remove_1000_indexed_orders() -> BenchResult {
        let user_count = COUNT / 10;
        add_users(0, user_count).unwrap();
        add_orders(0, COUNT, user_count).unwrap();
        create_indices();

        bench_fn(|| {
            execute("DELETE FROM orders WHERE user_id <= 100 ");
        })
    }

    #[bench(raw)]
    fn bench_create_1000000_indexed_orders() -> BenchResult {
        let user_count = COUNT / 10;
        add_users(0, user_count).unwrap();
        add_orders(0, COUNT, user_count).unwrap();
        create_indices();
        execute("DELETE FROM orders");

        bench_fn(|| {
            add_orders(0, COUNT, user_count).unwrap();
        })
    }

    #[bench(raw)]
    fn bench_delete_100000_indexed_orders_and_rollback() -> BenchResult {
        let user_count = COUNT / 10;
        add_users(0, user_count).unwrap();
        add_orders(0, COUNT, user_count).unwrap();
        create_indices();

        let res = query("SELECT COUNT(*) FROM orders".to_string()).unwrap();
        let s = res[0][0].clone().unwrap();
        let cnt: i64 = s.parse().expect("Not a valid number");

        assert_eq!(cnt, 1000000);

        let result = bench_fn(|| {
            execute("BEGIN TRANSACTION");
            execute("DELETE FROM orders WHERE order_id > 900000");

            let res = query("SELECT COUNT(*) FROM orders".to_string()).unwrap();
            let s = res[0][0].clone().unwrap();
            let cnt: i64 = s.parse().expect("Not a valid number");

            assert_eq!(cnt, 900000);

            execute("ROLLBACK");
        });

        let res = query("SELECT COUNT(*) FROM orders".to_string()).unwrap();
        let s = res[0][0].clone().unwrap();
        let cnt: i64 = s.parse().expect("Not a valid number");

        assert_eq!(cnt, 1000000);

        result
    }
}
