use std::cell::RefCell;

use candid::CandidType;
use candid::Deserialize;
use rusqlite::types::Type;
use rusqlite::Connection;
use rusqlite::ToSql;

use ic_stable_structures::memory_manager::MemoryId;
use ic_stable_structures::{memory_manager::MemoryManager, DefaultMemoryImpl};

thread_local! {
    static DB: RefCell<Option<Connection>> = const { RefCell::new(None) };
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

type Result<T = String, E = Error> = std::result::Result<T, E>;

type QueryResult<T = Vec<Vec<Option<String>>>, E = Error> = std::result::Result<T, E>;

const MOUNTED_MEMORY_ID: u8 = 20;
const DB_FILE_NAME: &str = "db.db3";
const JOURNAL_NAME: &str = "db.db3-journal";

fn setup_runtime() {
    init_polyfill();
    mount_memory_files();
    open_database();
    set_pragmas();
}

fn set_pragmas() {
    // set pragmas
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        // persist journal file. Setting to `OFF` will work faster but the atomic COMMIT/ROLLBACK operations will not work (see documentation).
        db.pragma_update(None, "journal_mode", &"PERSIST" as &dyn ToSql)
            .unwrap();

        // reduce synchronizations.
        db.pragma_update(None, "synchronous", &"NORMAL" as &dyn ToSql)
            .unwrap();

        // use fewer writes to disk with larger memory chunks.
        // Larger page size (say 16384) gives about 10% performance improvement when adding large batches of new records.
        // Can slow down up to 30% for database changes scattered accross its memory.
        // (any small change causes the sqlite to rewrite the whole page)
        db.pragma_update(None, "page_size", &4096 as &dyn ToSql)
            .unwrap();

        // reduce locks and unlocks, since the canister is the only user of the database with no concurrent connections,
        // there is no need to lock and unlock the database for each of the queries.
        // Note: For this mode it is important that the database is unlocked before upgrading the canister
        // by explicitly destroying the connection in the pre_upgrade hook, otherwise the lock file
        // will be present after upgrade, and it won't be possible to open a new connection later on.
        db.pragma_update(None, "locking_mode", &"EXCLUSIVE" as &dyn ToSql)
            .unwrap();

        // temp_store = MEMORY, this disables creating temp files on the disk during complex queries,
        // this workaround is currently necessary to avoid the error when sqlite tries to create a temporary file and breaks
        db.pragma_update(None, "temp_store", &"MEMORY" as &dyn ToSql)
            .unwrap();

        // Add this option to minimize disk reads and work in canister memory instead.
        // Some operations like batch insertions can have lower performance with this option.
        // Some operations related to adding indexed records have better performance.
        db.pragma_update(None, "cache_size", &1000000 as &dyn ToSql)
            .unwrap();
    });
}

fn mount_memory_files() {
    MEMORY_MANAGER.with(|m| {
        let m = m.borrow();

        // mount virtual memory as file for faster DB operations
        // This ensures 2% to 40% performance improvement
        ic_wasi_polyfill::mount_memory_file(
            DB_FILE_NAME,
            Box::new(m.get(MemoryId::new(MOUNTED_MEMORY_ID))),
        );

        // 5 - 7% performance improvement on some operations
        ic_wasi_polyfill::mount_memory_file(
            JOURNAL_NAME,
            Box::new(m.get(MemoryId::new(MOUNTED_MEMORY_ID + 1))),
        );
    });
}

#[ic_cdk::query]
fn query(sql: String) -> QueryResult {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let mut stmt = db.prepare(&sql).unwrap();
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
                        message: format!("{:?}", err),
                    })
                }
            }
        }
        Ok(res)
    })
}

fn execute(sql: &str) {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();
        db.execute(sql, ()).unwrap();
    });
}

fn init_polyfill() {
    MEMORY_MANAGER.with(|m| {
        let m = m.borrow();
        ic_wasi_polyfill::init_with_memory_manager(&[0u8; 32], &[], &m, 200..210);
    });
}

fn open_database() {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        *db = Some(Connection::open(DB_FILE_NAME).unwrap());
    });
}

fn close_database() {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        *db = None;
    });
}

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

fn create_indices() {
    execute("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);");
    execute("CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id);");
}

#[ic_cdk::init]
fn init() {
    setup_runtime();
    create_tables();
}

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    close_database();
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    setup_runtime();
}

#[derive(CandidType, Deserialize, Debug)]
enum Error {
    InvalidCanister,
    CanisterError { message: String },
}

#[ic_cdk::update]
fn add_users(offset: usize, count: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();
        let tx = db.transaction().unwrap();

        let sql = String::from("insert into users (username, email) values (?, ?)");

        {
            let mut stmt = tx.prepare_cached(&sql).unwrap();

            let mut i = 0;

            while i < count {
                let id = offset + i + 1;
                let username = format!("user{}", id);
                let email = format!("user{}@example.com", id);

                stmt.execute(rusqlite::params![username, email])
                    .expect("insert of a user failed!");

                i += 1;
            }
        }

        tx.commit().expect("COMMIT USER INSERTION FAILED!");

        Ok(String::from("bench1_insert_person OK"))
    })
}

#[ic_cdk::update]
fn add_orders(offset: usize, count: usize, id_mod: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();
        let tx = db.transaction().unwrap();

        let sql = String::from("insert into orders (user_id, amount) values (?, ?)");

        {
            let mut stmt = tx.prepare_cached(&sql).unwrap();

            let mut i = 0;

            while i < count {
                let id = (offset + i + 1) * 13 % id_mod + 1;

                stmt.execute(rusqlite::params![id, (id * 100 + id * 17) / 15])
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

    const COUNT: usize = 1000000usize;

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
