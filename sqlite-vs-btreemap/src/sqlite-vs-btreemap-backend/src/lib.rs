use candid::CandidType;
use candid::Deserialize;

use ic_rusqlite::types::Type;
use ic_rusqlite::with_connection;
use ic_stable_structures::memory_manager::MemoryManager;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::BTreeMap;
use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::Memory;
use ic_stable_structures::StableBTreeMap;

use ic_stable_structures::memory_manager::MemoryId;
use std::cell::RefCell;

#[derive(CandidType, Deserialize, Debug)]
enum Error {
    InvalidCanister,
    CanisterError { message: String },
}

type Result<T = String, E = Error> = std::result::Result<T, E>;

type QueryResult<T = Vec<Vec<Option<String>>>, E = Error> = std::result::Result<T, E>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static MAP: RefCell<BTreeMap<u64, String, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        BTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    static PAYLOAD: RefCell<String> = RefCell::new(String::new());
}

fn init_payload(size: usize) {
    PAYLOAD.with(|payload| {
        let mut payload = payload.borrow_mut();
        let cap = payload.capacity();

        if cap < size {
            payload.reserve(size - cap);
        }

        payload.clear();

        payload.extend(std::iter::repeat('a').take(size));
    });
}

#[ic_cdk::update]
fn create_tables() {
    with_connection(|mut conn| {
        conn.execute("CREATE TABLE IF NOT EXISTS users ( id INTEGER PRIMARY KEY AUTOINCREMENT, username TEXT NOT NULL)", ());
    });
}

#[ic_cdk::init]
fn init() {
    create_tables();
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    init();
}

fn for_each_user(offset: u64, increment: u64, count: u64, mut f: impl FnMut(u64)) {
    let mut i = 0;
    while i < count {
        let id = offset + i;
        f(id);
        i += increment;
    }
}

#[ic_cdk::update]
fn add_users_btree(offset: u64, increment: u64, count: u64) {
    PAYLOAD.with_borrow(|payload| {
        MAP.with_borrow_mut(|map| {
            for_each_user(offset, increment, count, |id| {
                map.insert(id, payload.clone());
            });
        })
    })
}

#[ic_cdk::update]
fn add_users_naive(offset: u64, increment: u64, count: u64) {
    PAYLOAD.with_borrow(|payload| {
        with_connection(|mut conn| {
            for_each_user(offset, increment, count, |id| {
                conn.execute(
                    "insert into users (id, username) values (?, ?);",
                    (&id, payload),
                );
            })
        })
    })
}

#[ic_cdk::update]
fn add_users_stored(offset: u64, increment: u64, count: u64) {
    PAYLOAD.with_borrow(|payload| {
        with_connection(|mut conn| {
            let mut i = 0;

            let mut stmt = conn
                .prepare("INSERT INTO users (id, username) VALUES (?1, ?2);")
                .expect("prepare failed");

            for_each_user(offset, increment, count, |id| {
                stmt.execute((&id, payload));
            });
        })
    })
}

#[ic_cdk::update]
fn add_users_bulk(offset: u64, increment: u64, count: u64) {
    PAYLOAD.with_borrow(|payload| {
        with_connection(|mut conn| {
            let tx = conn.transaction().unwrap();
            let sql = String::from("insert into users (id, username) values (?, ?);");

            {
                let mut stmt = tx.prepare_cached(&sql).unwrap();

                let mut i = 0;

                for_each_user(offset, increment, count, |id| {
                    stmt.execute((id, payload))
                        .expect("insert of a user failed!");
                });
            }

            tx.commit().expect("COMMIT USER INSERTION FAILED!");
        })
    })
}

mod benches {
    use super::*;
    use canbench_rs::{bench, bench_fn, BenchResult};

    const INITIAL_COUNT: u64 = 100000u64;

    const OFFSET: u64 = INITIAL_COUNT / 2 + 5;
    const COUNT: u64 = 1000u64;
    const PAYLOAD_SIZE: usize = 1000usize;

    #[bench(raw)]
    fn bench_add_users_btree() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        add_users_btree(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_btree(OFFSET, 10, COUNT);
        })
    }

    /*
    #[bench(raw)]
    fn bench_add_users_naive() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        add_users_naive(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_naive(OFFSET, 10, COUNT);
        })
    }

    #[bench(raw)]
    fn bench_add_users_stored() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        add_users_stored(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_stored(OFFSET, 10, COUNT);
        })
    }
    */

    #[bench(raw)]
    fn bench_add_users_bulk() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        add_users_bulk(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_bulk(OFFSET, 10, COUNT);
        })
    }

    #[bench(raw)]
    fn bench_read_users_btree() -> BenchResult {
        // Prepopulate
        init_payload(PAYLOAD_SIZE);
        add_users_btree(OFFSET, 10, COUNT);
        add_users_btree(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            MAP.with_borrow(|map| {
                for_each_user(OFFSET, 10, COUNT, |id| {
                    let _ = map.get(&id);
                });
            });
        })
    }

    #[bench(raw)]
    fn bench_read_users_bulk() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        add_users_bulk(OFFSET, 10, COUNT);
        add_users_bulk(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            with_connection(|mut conn| {
                let tx = conn.transaction().unwrap();
                let mut stmt = tx
                    .prepare_cached("SELECT username FROM users WHERE id = ?1")
                    .unwrap();
                for_each_user(OFFSET, 10, COUNT, |id| {
                    let _: Result<Vec<String>, _> =
                        stmt.query_map((&id,), |row| row.get(0)).unwrap().collect();
                });
            });
        })
    }
}
