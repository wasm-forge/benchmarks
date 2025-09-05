use ic_stable_structures::memory_manager::MemoryManager;
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;
use std::fs;

thread_local! {
    // The memory manager enables multiple virtual memories in one.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

#[ic_cdk::query]
fn greet(name: String) -> String {
    let message = format!("Hello from WASI: {name}");

    println!("{}", message);

    message
}

#[ic_cdk::init]
fn init() {
    MEMORY_MANAGER.with(|m| {
        let m = m.borrow();
        ic_wasi_polyfill::init_with_memory_manager(&[0u8; 32], &[], &m, 101..119);
    });
}

pub fn create_folders(dirname: String, count: u32) {
    // Loop to create directories
    for i in 0..count {
        let dname = format!("{dirname}{i}");
        std::fs::create_dir_all(&dname).expect("Failed to create directory");
    }
}

pub fn list_folders(path: String) -> Vec<String> {
    let mut folder_names = Vec::new();

    // get all the files and folders and return their names
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                // Push the folder name to the vector
                if let Some(name) = entry.file_name().to_str() {
                    folder_names.push(name.to_string());
                }
            }
        }
    }

    folder_names
}

mod benches {
    use super::*;
    use canbench_rs::{bench, bench_fn, BenchResult};

    #[bench(raw)]
    fn create_1000_folders() -> BenchResult {
        let file_name = "dir";

        let res = bench_fn(|| {
            // bench
            create_folders(file_name.to_string(), 1000);
        });

        res
    }

    #[bench(raw)]
    fn create_1000_folders_1000_subfolders() -> BenchResult {
        let file_name = "dir";
        let file_name2 = "dir999/dir";
        create_folders(file_name.to_string(), 1000);

        let res = bench_fn(|| {
            // bench
            create_folders(file_name2.to_string(), 1000);
        });

        res
    }

    #[bench(raw)]
    fn list_1000_folders() -> BenchResult {
        let file_name = "dir";

        create_folders(file_name.to_string(), 1000);

        let res = bench_fn(|| {
            // bench
            list_folders(".".to_string());
        });

        res
    }
}
