use std::cell::RefCell;
use std::ptr::null;

use candid::CandidType;
use candid::Deserialize;
use rusqlite::types::Type;
use rusqlite::Connection;
use rusqlite::ToSql;

use ic_stable_structures::memory_manager::MemoryId;
use ic_stable_structures::{memory_manager::MemoryManager, DefaultMemoryImpl};

thread_local! {
    static DB: RefCell<Option<Connection>> = RefCell::new(None);
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

type Result<T = String, E = Error> = std::result::Result<T, E>;

type QueryResult<T = Vec<Vec<String>>, E = Error> = std::result::Result<T, E>;

const MOUNTED_MEMORY_ID: u8 = 20;
const DB_FILE_NAME: &str = "db.db3";

#[ic_cdk::update]
fn add(name: String, data: String, age: u32) {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();
        db.execute(
            "INSERT INTO person (name, data, age) VALUES (?1, ?2, ?3)",
            (&name, &data, age),
        )
        .unwrap();
    });
}

#[ic_cdk::query]
fn list() -> Vec<(u64, String, String, u32)> {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();
        let mut stmt = db
            .prepare("SELECT id, name, data, age FROM person")
            .unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0).unwrap(),
                    row.get(1).unwrap(),
                    row.get(2).unwrap(),
                    row.get(3).unwrap(),
                ))
            })
            .unwrap();
        let mut result = vec![];
        for person in rows {
            result.push(person.unwrap());
        }
        result
    })
}

#[ic_cdk::query]
fn query(sql: String) -> QueryResult {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let mut stmt = db.prepare(&sql).unwrap();
        let cnt = stmt.column_count();
        let mut rows = stmt.query([]).unwrap();
        let mut res: Vec<Vec<String>> = Vec::new();

        loop {
            match rows.next() {
                Ok(row) => match row {
                    Some(row) => {
                        let mut vec: Vec<String> = Vec::new();
                        for idx in 0..cnt {
                            let v = row.get_ref_unwrap(idx);
                            match v.data_type() {
                                Type::Null => vec.push(String::from("")),
                                Type::Integer => vec.push(v.as_i64().unwrap().to_string()),
                                Type::Real => vec.push(v.as_f64().unwrap().to_string()),
                                Type::Text => vec.push(v.as_str().unwrap().parse().unwrap()),
                                Type::Blob => vec.push(hex::encode(v.as_blob().unwrap())),
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

fn mount_memory_files() {
    MEMORY_MANAGER.with(|m| {
        let m = m.borrow();
        ic_wasi_polyfill::init_with_memory_manager(&[0u8; 32], &[], &m, 200..210);

        // mount virtual memory as file for faster DB operations
        let memory = m.get(MemoryId::new(MOUNTED_MEMORY_ID));
        ic_wasi_polyfill::mount_memory_file(DB_FILE_NAME, Box::new(memory));
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

fn set_pragmas() {
    // set pragmas
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        // do not create and destroy the journal file every time, set its size to 0 instead.
        // This option works faster for normal files,
        // this option is also mandatory, if you are using mounted memory files as a storage.
        db.pragma_update(None, "journal_mode", &"TRUNCATE" as &dyn ToSql)
            .unwrap();

        // use fewer writes to disk with larger memory chunks.
        // This pragma gives about 10% improvement when adding large batches of new records.
        // Can slow down up to 30% for operations that change the database in different places.
        // (any small change will cause the DB server to rewrite the whole DB page)
        //db.pragma_update(None, "page_size", &16384 as &dyn ToSql)
        //    .unwrap();

        // reduce locks and unlocks, since the canister in the only user of the database with no concurrent connections,
        // there is no need to lock and unlock the database for each of the queries.
        db.pragma_update(None, "locking_mode", &"EXCLUSIVE" as &dyn ToSql)
            .unwrap();

        // temp_store = MEMORY, disables creating temp files during complex queries, improves performance,
        // this workaround is currently necessary to avoid error when creating a new tmp file
        db.pragma_update(None, "temp_store", &2 as &dyn ToSql)
            .unwrap();

        // Add this option to minimize disk reads and work in canister memory instead.
        // Some operations, like batch insertions can have lower performance with this option.
        // Some operations related to adding indexed records have better performance.
        db.pragma_update(None, "cache_size", &1000000 as &dyn ToSql)
            .unwrap();
    });
}

#[ic_cdk::init]
fn init() {
    mount_memory_files();

    open_database();
    set_pragmas();
    create_tables();
}

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    close_database();
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    mount_memory_files();

    open_database();
    set_pragmas();
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
                    .expect("INSERT USER FAILED!");

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
                    .expect("INSERT USER FAILED!");

                i += 1;
            }
        }

        tx.commit().expect("COMMIT USER INSERTION FAILED!");

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
    fn bench_remove_100_indexed_orders() -> BenchResult {
        let user_count = COUNT / 10;
        add_users(0, user_count).unwrap();
        add_orders(0, COUNT, user_count).unwrap();
        create_indices();

        bench_fn(|| {
            execute("DELETE FROM orders WHERE user_id <= 100 ");
        })
    }

    #[bench(raw)]
    fn bench_create_indexed_orders() -> BenchResult {
        let user_count = COUNT / 10;
        add_users(0, user_count).unwrap();
        add_orders(0, COUNT, user_count).unwrap();
        create_indices();
        execute("DELETE FROM orders");

        bench_fn(|| {
            add_orders(0, COUNT, user_count).unwrap();
        })
    }
}
