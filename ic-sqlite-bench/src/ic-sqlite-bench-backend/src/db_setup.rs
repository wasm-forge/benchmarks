use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

use candid::CandidType;
use candid::Deserialize;
use lazy_static::lazy_static;
use rusqlite::Connection;
use rusqlite::ToSql;

use ic_stable_structures::memory_manager::MemoryId;
use ic_stable_structures::{memory_manager::MemoryManager, DefaultMemoryImpl};

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

lazy_static! {
    pub static ref CONN: Arc<Mutex<Connection>> = {
        let conn = Connection::open(DB_FILE_NAME).unwrap();
        return Arc::new(Mutex::new(conn));
    };
}

type Result<T = String, E = Error> = std::result::Result<T, E>;

type QueryResult<T = Vec<Vec<Option<String>>>, E = Error> = std::result::Result<T, E>;

const MOUNTED_MEMORY_ID: u8 = 20;
const DB_FILE_NAME: &str = "db.db3";
const JOURNAL_NAME: &str = "db.db3-journal";
const WAL_NAME: &str = "db.db3-wal";

#[ic_cdk::init]
pub fn init() {
    setup_runtime();
}

#[ic_cdk::pre_upgrade]
pub fn pre_upgrade() {
    //...
}

#[ic_cdk::post_upgrade]
pub fn post_upgrade() {
    setup_runtime();
}

#[derive(CandidType, Deserialize, Debug)]
enum Error {
    InvalidCanister,
    CanisterError { message: String },
}

fn setup_runtime() {
    init_polyfill();

    mount_memory_files();
    set_pragmas();
}

fn set_pragmas() {
    // set pragmas
    let db = CONN.lock().unwrap();

    // do not create and destroy the journal file every time, set its size to 0 instead.
    // This option works faster for normal files,
    // this option is mandatory, if you are using mounted memory files as a storage.
    db.pragma_update(None, "journal_mode", &"MEMORY" as &dyn ToSql)
        .unwrap();

    // reduce synchronizations
    db.pragma_update(None, "synchronous", &0 as &dyn ToSql)
        .unwrap();

    // use fewer writes to disk with larger memory chunks.
    // This pragma gives about 10% performance improvement when adding large batches of new records.
    // Can slow down up to 30% for database changes scattered accross its memory.
    // (any small change will cause the sqlite to rewrite the whole page)
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
    // this workaround is currently necessary to avoid error when sqlite tries to create a temporary file
    db.pragma_update(None, "temp_store", &2 as &dyn ToSql)
        .unwrap();

    // Add this option to minimize disk reads and work in canister memory instead.
    // Some operations like batch insertions can have lower performance with this option.
    // Some operations related to adding indexed records have better performance.
    //db.pragma_update(None, "cache_size", &1000000 as &dyn ToSql)
    //    .unwrap();
}

fn mount_memory_files() {
    MEMORY_MANAGER.with(|m| {
        let m = m.borrow();

        // mount virtual memory as file for faster DB operations
        ic_wasi_polyfill::mount_memory_file(
            DB_FILE_NAME,
            Box::new(m.get(MemoryId::new(MOUNTED_MEMORY_ID))),
        );
        /*
        ic_wasi_polyfill::mount_memory_file(
            JOURNAL_NAME,
            Box::new(m.get(MemoryId::new(MOUNTED_MEMORY_ID + 1))),
        );
        ic_wasi_polyfill::mount_memory_file(
            WAL_NAME,
            Box::new(m.get(MemoryId::new(MOUNTED_MEMORY_ID + 2))),
        );
        */
    });
}

fn init_polyfill() {
    MEMORY_MANAGER.with(|m| {
        let m = m.borrow();
        ic_wasi_polyfill::init_with_memory_manager(&[0u8; 32], &[], &m, 200..210);
    });
}
