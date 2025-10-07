use candid::CandidType;
use candid::Deserialize;

use ic_rusqlite::with_connection;
use ic_stable_structures::memory_manager::MemoryManager;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::BTreeMap;
use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::Memory;

use ic_stable_structures::memory_manager::MemoryId;
use std::cell::RefCell;

#[derive(CandidType, Deserialize, Debug)]
enum Error {
    InvalidCanister,
    CanisterError { message: String },
}

type Result<T = String, E = Error> = std::result::Result<T, E>;

const PROFILING: MemoryId = MemoryId::new(50);

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
    with_connection(|conn| {
        conn.execute("CREATE TABLE IF NOT EXISTS users ( id INTEGER PRIMARY KEY AUTOINCREMENT, username TEXT NOT NULL)", ()).unwrap();
    });
}

pub fn profiling_init() {
    let memory = MEMORY_MANAGER.with(|m| m.borrow().get(PROFILING));
    memory.grow(4096);
}

#[ic_cdk::init]
fn init() {
    profiling_init();
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
fn add_users_sqlite(offset: u64, increment: u64, count: u64) {
    PAYLOAD.with_borrow(|payload| {
        with_connection(|mut conn| {
            let tx = conn.transaction().unwrap();
            let sql = String::from("insert into users (id, username) values (?, ?);");

            {
                let mut stmt = tx.prepare_cached(&sql).unwrap();

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

    // initial count of inserted elements
    const INITIAL_COUNT: u64 = 1000000u64;
    // do insertions in the middle of the existing element set
    const OFFSET: u64 = INITIAL_COUNT / 2 + 5;
    // number of elements to insert or to read
    const COUNT: u64 = 1u64;
    // size of an element being inserted
    const PAYLOAD_SIZE: usize = 100usize;

    #[bench(raw)]
    fn bench_btree_add_00001() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        add_users_btree(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_btree(OFFSET, 10, 1);
        })
    }

    #[bench(raw)]
    fn bench_btree_add_00010() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        add_users_btree(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_btree(OFFSET, 10, 10);
        })
    }

    #[bench(raw)]
    fn bench_btree_add_00050() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        add_users_btree(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_btree(OFFSET, 10, 50);
        })
    }

    #[bench(raw)]
    fn bench_btree_add_00100() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        add_users_btree(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_btree(OFFSET, 10, 100);
        })
    }

    #[bench(raw)]
    fn bench_btree_add_01000() -> BenchResult {
        init_payload(PAYLOAD_SIZE);

        add_users_btree(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_btree(OFFSET, 10, 1000);
        })
    }

    #[bench(raw)]
    fn bench_btree_add_10000() -> BenchResult {
        init_payload(PAYLOAD_SIZE);

        add_users_btree(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_btree(OFFSET, 10, 10000);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_add_00001() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        create_tables();

        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 1);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_add_00010() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        create_tables();

        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 10);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_add_00050() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        create_tables();

        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 50);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_add_00100() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        create_tables();

        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 100);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_add_01000() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        create_tables();

        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 1000);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_add_10000() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        create_tables();

        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 10000);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_memory_journal_add_00001() -> BenchResult {
        let mut config = ic_rusqlite::ConnectionConfig::new();

        config
            .pragma_settings
            .insert("journal_mode".to_string(), "MEMORY".to_string());
        ic_rusqlite::set_connection_config(config);
        create_tables();

        init_payload(PAYLOAD_SIZE);
        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 1);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_memory_journal_add_00010() -> BenchResult {
        let mut config = ic_rusqlite::ConnectionConfig::new();

        config
            .pragma_settings
            .insert("journal_mode".to_string(), "MEMORY".to_string());
        ic_rusqlite::set_connection_config(config);
        create_tables();

        init_payload(PAYLOAD_SIZE);
        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 10);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_memory_journal_add_00050() -> BenchResult {
        let mut config = ic_rusqlite::ConnectionConfig::new();

        config
            .pragma_settings
            .insert("journal_mode".to_string(), "MEMORY".to_string());
        ic_rusqlite::set_connection_config(config);
        create_tables();

        init_payload(PAYLOAD_SIZE);
        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 50);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_memory_journal_add_00100() -> BenchResult {
        let mut config = ic_rusqlite::ConnectionConfig::new();

        config
            .pragma_settings
            .insert("journal_mode".to_string(), "MEMORY".to_string());
        ic_rusqlite::set_connection_config(config);

        init_payload(PAYLOAD_SIZE);
        create_tables();

        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 100);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_memory_journal_add_01000() -> BenchResult {
        let mut config = ic_rusqlite::ConnectionConfig::new();

        config
            .pragma_settings
            .insert("journal_mode".to_string(), "MEMORY".to_string());
        ic_rusqlite::set_connection_config(config);

        init_payload(PAYLOAD_SIZE);
        create_tables();

        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 1000);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_memory_journal_add_10000() -> BenchResult {
        let mut config = ic_rusqlite::ConnectionConfig::new();

        config
            .pragma_settings
            .insert("journal_mode".to_string(), "MEMORY".to_string());
        ic_rusqlite::set_connection_config(config);

        init_payload(PAYLOAD_SIZE);
        create_tables();

        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 10000);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_no_journal_add_00001() -> BenchResult {
        let mut config = ic_rusqlite::ConnectionConfig::new();

        config
            .pragma_settings
            .insert("journal_mode".to_string(), "OFF".to_string());
        ic_rusqlite::set_connection_config(config);
        create_tables();

        init_payload(PAYLOAD_SIZE);
        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 1);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_no_journal_add_00010() -> BenchResult {
        let mut config = ic_rusqlite::ConnectionConfig::new();

        config
            .pragma_settings
            .insert("journal_mode".to_string(), "OFF".to_string());
        ic_rusqlite::set_connection_config(config);
        create_tables();

        init_payload(PAYLOAD_SIZE);
        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 10);
        })
    }
    #[bench(raw)]
    fn bench_sqlite_no_journal_add_00050() -> BenchResult {
        let mut config = ic_rusqlite::ConnectionConfig::new();

        config
            .pragma_settings
            .insert("journal_mode".to_string(), "OFF".to_string());
        ic_rusqlite::set_connection_config(config);
        create_tables();

        init_payload(PAYLOAD_SIZE);
        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 50);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_no_journal_add_00100() -> BenchResult {
        let mut config = ic_rusqlite::ConnectionConfig::new();

        config
            .pragma_settings
            .insert("journal_mode".to_string(), "OFF".to_string());
        ic_rusqlite::set_connection_config(config);

        init_payload(PAYLOAD_SIZE);
        create_tables();

        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 100);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_no_journal_add_01000() -> BenchResult {
        let mut config = ic_rusqlite::ConnectionConfig::new();

        config
            .pragma_settings
            .insert("journal_mode".to_string(), "OFF".to_string());
        ic_rusqlite::set_connection_config(config);

        init_payload(PAYLOAD_SIZE);
        create_tables();

        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 1000);
        })
    }

    #[bench(raw)]
    fn bench_sqlite_no_journal_add_10000() -> BenchResult {
        let mut config = ic_rusqlite::ConnectionConfig::new();

        config
            .pragma_settings
            .insert("journal_mode".to_string(), "OFF".to_string());
        ic_rusqlite::set_connection_config(config);

        init_payload(PAYLOAD_SIZE);
        create_tables();

        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            add_users_sqlite(OFFSET, 10, 10000);
        })
    }

    /*
    #[bench(raw)]
    fn bench_read_users_btree() -> BenchResult {
        // Prepopulate
        init_payload(PAYLOAD_SIZE);
        create_tables();

        add_users_btree(OFFSET, 100, COUNT);
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
    fn bench_read_users_sqlite() -> BenchResult {
        init_payload(PAYLOAD_SIZE);
        create_tables();

        add_users_sqlite(OFFSET, 10, COUNT);
        close_connection(); // clear cache
        add_users_sqlite(0, 10, INITIAL_COUNT);

        bench_fn(|| {
            with_connection(|conn| {
                let mut stmt = conn
                    .prepare_cached("SELECT username FROM users WHERE id = ?1")
                    .unwrap();

                for_each_user(OFFSET, 10, COUNT, |id| {
                    let _row: String = stmt.query_row((&id,), |row| row.get(0)).unwrap();
                });
            });
        })
    }
    */
}
