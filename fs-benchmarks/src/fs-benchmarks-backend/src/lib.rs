use ic_stable_structures::memory_manager::MemoryId;
use ic_stable_structures::memory_manager::MemoryManager;
use ic_stable_structures::DefaultMemoryImpl;

use stable_fs::fs::DstBuf;
use stable_fs::fs::Fd;
use stable_fs::fs::FdStat;
use stable_fs::fs::FileSystem;
use stable_fs::fs::OpenFlags;
use stable_fs::fs::SrcBuf;
use stable_fs::fs::Whence;

use stable_fs::error::Error;

use stable_fs::storage::stable::StableStorage;
use stable_fs::storage::types::FileType;

use std::cell::RefCell;

const SEGMENT_SIZE: usize = 1000usize;
const FILES_COUNT: usize = 10usize;

const USE_MOUNTED_MEMORY: bool = false;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static FS: RefCell<FileSystem> = {

        MEMORY_MANAGER.with(|m| {

            let memory_manager = m.borrow();

            //v0.4
            let storage = StableStorage::new_with_memory_manager(&memory_manager, 200u8);
            //v0.5, v0.6 ...
            //let mut storage = StableStorage::new_with_memory_manager(&memory_manager, 200..210u8);

            // set chunk version to V1
            //storage.set_chunk_type(storage::stable::ChunkType::V1);

            // setup chunk size
            //storage.set_chunk_size(stable_fs::fs::ChunkSize::CHUNK4K).unwrap();
            //storage.set_chunk_size(stable_fs::fs::ChunkSize::CHUNK64K).unwrap();

            let fs = RefCell::new(
                FileSystem::new(Box::new(storage)).unwrap()
            );

            // use mounted memory
            if USE_MOUNTED_MEMORY {
                let filename = "file.txt";

                //fs.borrow_mut().mount_memory_file(filename, Box::new(memory_manager.get(MemoryId::new(15)))).unwrap();

                for i in 0..FILES_COUNT {
                //    fs.borrow_mut().mount_memory_file(&format!("{}{}", filename, i), Box::new(memory_manager.get(MemoryId::new(15 + i as u8)))).unwrap();
                }
            }

            fs
        })
    };
}

fn open_file(
    fs: &mut FileSystem,
    root_fd: Fd,
    filename: &str,
    fdstat: FdStat,
    open_flags: OpenFlags,
    ctime: u64,
) -> Result<Fd, Error> {
    // v0.7
    //fs.open(root_fd, filename, fdstat, open_flags, ctime)
    // below v0.7
    fs.open_or_create(root_fd, filename, fdstat, open_flags, ctime)
}

fn file_size(filename: String) -> usize {
    FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        let dir = fs.root_fd();

        let fd = open_file(
            &mut fs,
            dir,
            filename.as_str(),
            FdStat::default(),
            OpenFlags::empty(),
            0,
        )
        .unwrap();

        let meta = fs.metadata(fd).unwrap();

        let size = meta.size;

        size as usize
    })
}

thread_local! {
    static BUFFER: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
}

pub fn instruction_counter() -> u64 {
    0
}

pub fn append_buffer(text: String, times: usize) -> usize {
    BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();

        if buffer.is_none() {
            *buffer = Some(Vec::new());
        }

        let buffer = buffer.as_mut().unwrap();

        for _ in 0..times {
            buffer.extend_from_slice(text.as_bytes());
        }

        buffer.len()
    })
}

pub fn check_buffer(text: String, times: usize) -> usize {
    BUFFER.with(|buffer| {
        let buffer = buffer.borrow_mut();

        let buffer = buffer.as_ref();

        if buffer.is_none() && times == 0 {
            return 0;
        }

        let buffer = buffer.unwrap();

        let mut p = 0;
        let len = text.len();

        let bytes = text.as_bytes();

        for _ in 0..times {
            let buf = &buffer[p..p + len];

            assert_eq!(bytes, buf);

            p += len;
        }

        assert_eq!(buffer.len(), text.len() * times);

        buffer.len()
    })
}

pub fn clear_buffer() {
    BUFFER.with(|chunk| {
        let mut chunk = chunk.borrow_mut();

        if chunk.is_none() {
            return;
        }

        let chunk = chunk.as_mut().unwrap();

        // explicitly fill contents with 0
        chunk.fill(0);

        chunk.clear()
    })
}

pub fn read_buffer(offset: usize, size: usize) -> String {
    BUFFER.with(|chunk| {
        let mut chunk = chunk.borrow_mut();

        let chunk = chunk.as_mut().unwrap();

        std::str::from_utf8(&chunk[offset..offset + size])
            .unwrap()
            .to_string()
    })
}

pub fn store_buffer(filename: String) -> usize {
    let res = BUFFER.with(|chunk| {
        let chunk = chunk.borrow_mut();

        let chunk = chunk.as_ref().unwrap();

        FS.with(|fs| {
            let mut fs = fs.borrow_mut();

            let root_fd = (*fs).root_fd();

            let fd = open_file(
                &mut fs,
                root_fd,
                &filename,
                FdStat::default(),
                OpenFlags::CREATE,
                42,
            )
            .unwrap();

            let write_content = [SrcBuf {
                buf: chunk.as_ptr(),
                len: chunk.len(),
            }];

            let res = (*fs).write_vec(fd, write_content.as_ref()).unwrap();

            (*fs).close(fd).unwrap();

            res as usize
        })
    });

    res
}

pub fn store_buffer_in_1000b_segments(filename: String) -> (u64, usize) {
    let stime = instruction_counter();

    let res = BUFFER.with(|chunk| {
        let chunk = chunk.borrow_mut();

        let chunk = chunk.as_ref().unwrap();

        FS.with(|fs| {
            let mut fs = fs.borrow_mut();

            let root_fd = (*fs).root_fd();

            let fd = open_file(
                &mut fs,
                root_fd,
                &filename,
                FdStat::default(),
                OpenFlags::CREATE,
                42,
            )
            .unwrap();

            (*fs).seek(fd, 0, Whence::SET).unwrap();

            let len = chunk.len();

            let mut p = 0;
            let part_len = SEGMENT_SIZE;
            let mut res = 0;

            while p < len {
                let write_len = (len - p).min(part_len);

                let write_content = [SrcBuf {
                    buf: chunk[p..(p + part_len).min(len)].as_ptr(),
                    len: write_len,
                }];

                res += (*fs).write_vec(fd, write_content.as_ref()).unwrap();

                p += write_len;
            }

            (*fs).close(fd).unwrap();

            res as usize
        })
    });

    let etime = instruction_counter();

    (etime - stime, res)
}

pub fn store_buffer_in_1000b_segments_10_files(filename: String) -> (u64, usize) {
    let stime = instruction_counter();

    let res = BUFFER.with(|chunk| {
        let chunk = chunk.borrow_mut();

        let chunk = chunk.as_ref().unwrap();

        FS.with(|fs| {
            let mut fs = fs.borrow_mut();

            let root_fd = (*fs).root_fd();

            let mut fds = Vec::<Fd>::new();

            for i in 0..FILES_COUNT {
                let fd = open_file(
                    &mut fs,
                    root_fd,
                    &format!("{}{}", filename, i),
                    FdStat::default(),
                    OpenFlags::CREATE,
                    42,
                )
                .unwrap();

                (*fs).seek(fd, 0, Whence::SET).unwrap();

                fds.push(fd);
            }

            let len = chunk.len();

            let mut p = 0;
            let part_len = SEGMENT_SIZE;
            let mut res = 0;
            let mut idx = 0;

            while p < len {
                let fd = fds[idx % FILES_COUNT];

                let write_len = (len - p).min(part_len);

                let write_content = [SrcBuf {
                    buf: chunk[p..(p + part_len).min(len)].as_ptr(),
                    len: write_len,
                }];

                res += (*fs).write_vec(fd, write_content.as_ref()).unwrap();

                p += write_len;

                idx += 1;
            }

            fds.iter_mut().for_each(|fd| (*fs).close(*fd).unwrap());

            res as usize
        })
    });

    let etime = instruction_counter();

    (etime - stime, res)
}

pub fn load_buffer(filename: String) -> (u64, usize) {
    let stime = instruction_counter();

    let res = BUFFER.with(|chunk| {
        let mut chunk = chunk.borrow_mut();

        let chunk = chunk.as_mut().unwrap();

        FS.with(|fs| {
            let mut fs = fs.borrow_mut();

            let root_fd = (*fs).root_fd();

            let fd = open_file(
                &mut fs,
                root_fd,
                &filename,
                FdStat::default(),
                OpenFlags::CREATE,
                42,
            )
            .unwrap();

            let size = (*fs).metadata(fd).unwrap().size as usize;

            (*fs).seek(fd, 0, Whence::SET).unwrap();

            let read_content = [DstBuf {
                buf: chunk.as_mut_ptr(),
                len: size,
            }];

            unsafe { chunk.set_len(size) };

            let res = (*fs).read_vec(fd, &read_content).unwrap();

            res as usize
        })
    });

    let etime = instruction_counter();

    (etime - stime, res)
}

pub fn load_buffer_in_1000b_segments(filename: String) -> (u64, usize) {
    let stime = instruction_counter();

    let res = BUFFER.with(|chunk| {
        let mut chunk = chunk.borrow_mut();

        let chunk = chunk.as_mut().unwrap();

        FS.with(|fs| {
            let mut fs = fs.borrow_mut();

            let root_fd = (*fs).root_fd();

            let fd = open_file(
                &mut fs,
                root_fd,
                &filename,
                FdStat::default(),
                OpenFlags::CREATE,
                42,
            )
            .unwrap();

            let len = (*fs).metadata(fd).unwrap().size as usize;

            (*fs).seek(fd, 0, Whence::SET).unwrap();

            let mut p = 0;
            let part_len = SEGMENT_SIZE;
            let mut res = 0;

            unsafe { chunk.set_len(len) };

            while p < len {
                let read_len = (len - p).min(part_len);

                let read_content = [DstBuf {
                    buf: chunk[p..p + read_len].as_mut_ptr(),
                    len: read_len,
                }];

                res += (*fs).read_vec(fd, read_content.as_ref()).unwrap();

                p += read_len;
            }

            res as usize
        })
    });

    let etime = instruction_counter();

    (etime - stime, res)
}

pub fn load_buffer_in_1000b_segments_10_files(filename: String) -> (u64, usize) {
    let stime = instruction_counter();

    let res = BUFFER.with(|chunk| {
        let mut chunk = chunk.borrow_mut();

        let chunk = chunk.as_mut().unwrap();

        FS.with(|fs| {
            let mut fs = fs.borrow_mut();

            let root_fd = (*fs).root_fd();

            let mut fds = Vec::<Fd>::new();

            for i in 0..FILES_COUNT {
                let fd = open_file(
                    &mut fs,
                    root_fd,
                    &format!("{}{}", filename, i),
                    FdStat::default(),
                    OpenFlags::CREATE,
                    42,
                )
                .unwrap();

                (*fs).seek(fd, 0, Whence::SET).unwrap();
                fds.push(fd);
            }

            let len = (*fs).metadata(fds[0]).unwrap().size as usize * FILES_COUNT;

            let mut p = 0;
            let part_len = SEGMENT_SIZE;
            let mut res = 0;

            unsafe { chunk.set_len(len) };

            let mut idx = 0;

            while p < len {
                let fd = fds[idx % FILES_COUNT];

                let read_len = (len - p).min(part_len);

                let read_content = [DstBuf {
                    buf: chunk[p..p + read_len].as_mut_ptr(),
                    len: read_len,
                }];

                res += (*fs).read_vec(fd, read_content.as_ref()).unwrap();

                p += read_len;

                assert!(read_len > 0, "read_len must be greated than 0");

                idx += 1;
            }

            res as usize
        })
    });

    let etime = instruction_counter();

    (etime - stime, res)
}

mod benches {
    use super::*;
    use canbench_rs::{bench, bench_fn, BenchResult};

    #[bench(raw)]
    fn write_100mb() -> BenchResult {
        let file_name = "file.txt";

        check_buffer("abc1234567".to_string(), 0);

        append_buffer("abc1234567".to_string(), 10_000_000);

        check_buffer("abc1234567".to_string(), 10_000_000);

        store_buffer("temp2.txt".to_string());

        // bench
        bench_fn(|| store_buffer(file_name.to_string()))
    }

    #[bench(raw)]
    fn write_100mb_over_existing() -> BenchResult {
        let file_name = "file.txt";

        append_buffer("abc1234567".to_string(), 10_000_000);

        store_buffer(file_name.to_string());
        store_buffer("temp2.txt".to_string());

        // bench
        bench_fn(|| store_buffer(file_name.to_string()))
    }

    #[bench(raw)]
    fn read_100mb() -> BenchResult {
        let file_name = "file.txt";

        append_buffer("abc1234567".to_string(), 10_000_000);
        store_buffer(file_name.to_string());
        store_buffer("temp2.txt".to_string());

        check_buffer("abc1234567".to_string(), 10_000_000);
        clear_buffer();
        check_buffer("abc1234567".to_string(), 0);

        // bench
        let res = bench_fn(|| load_buffer(file_name.to_string()));

        check_buffer("abc1234567".to_string(), 10_000_000);

        res
    }

    #[bench(raw)]
    fn write_100mb_in_segments() -> BenchResult {
        let file_name = "file.txt";

        append_buffer("abc1234567".to_string(), 10_000_000);

        store_buffer("temp1.txt".to_string());

        //bench
        let res = bench_fn(|| store_buffer_in_1000b_segments(file_name.to_string()));

        assert_eq!(file_size(file_name.to_string()), 100_000_000);
        clear_buffer();
        load_buffer(file_name.to_string());
        check_buffer("abc1234567".to_string(), 10_000_000);

        res
    }

    #[bench(raw)]
    fn write_100mb_in_segments_over_existing() -> BenchResult {
        let file_name = "file.txt";

        append_buffer("abc1234567".to_string(), 10_000_000);

        store_buffer("temp1.txt".to_string());
        store_buffer(file_name.to_string());
        store_buffer("temp2.txt".to_string());

        // bench
        bench_fn(|| store_buffer_in_1000b_segments(file_name.to_string()))
    }

    #[bench(raw)]
    fn read_100mb_in_segments() -> BenchResult {
        let file_name = "file.txt";

        append_buffer("abc1234567".to_string(), 10_000_000);

        store_buffer("temp1.txt".to_string());
        store_buffer(file_name.to_string());
        store_buffer("temp2.txt".to_string());

        clear_buffer();
        check_buffer("abc1234567".to_string(), 0);

        //bench
        let res = bench_fn(|| load_buffer_in_1000b_segments(file_name.to_string()));

        check_buffer("abc1234567".to_string(), 10_000_000);

        res
    }

    #[bench(raw)]
    fn write_100mb_in_segments_10_files() -> BenchResult {
        let file_name = "file.txt";

        append_buffer("abc1234567".to_string(), 10_000_000);
        //store_buffer("temp1.txt".to_string());

        // bench
        let res = bench_fn(|| store_buffer_in_1000b_segments_10_files(file_name.to_string()));

        clear_buffer();
        load_buffer_in_1000b_segments_10_files(file_name.to_string());
        check_buffer("abc1234567".to_string(), 10_000_000);

        res
    }

    #[bench(raw)]
    fn write_100mb_in_segments_over_existing_10_files() -> BenchResult {
        let file_name = "file.txt";

        append_buffer("abc1234567".to_string(), 10_000_000);

        //store_buffer("temp1.txt".to_string());
        store_buffer_in_1000b_segments_10_files(file_name.to_string());
        store_buffer("temp2.txt".to_string());

        // bench
        bench_fn(|| bench_fn(|| store_buffer_in_1000b_segments_10_files(file_name.to_string())))
    }

    #[bench(raw)]
    fn read_100mb_in_segments_from_10_files() -> BenchResult {
        let file_name = "file.txt";

        append_buffer("abc1234567".to_string(), 10_000_000);

        //        store_buffer("temp1.txt".to_string());
        store_buffer_in_1000b_segments_10_files(file_name.to_string());
        //        store_buffer("temp2.txt".to_string());

        clear_buffer();

        let res = bench_fn(|| {
            // bench
            load_buffer_in_1000b_segments_10_files(file_name.to_string());
        });

        check_buffer("abc1234567".to_string(), 10_000_000);

        res
    }
}
