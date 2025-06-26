use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use rand::Rng;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Instant;

// use ratatui::{
//     backend::CrosstermBackend,
//     prelude::*,
//     widgets::{Block, Borders, Cell, Row, Table},
// };
// use tempfile::tempfile;

const SIZES: &[(usize, &str)] = &[
    (4 * 1024, "4k"),
    (64 * 1024, "64k"),
    (512 * 1024, "512k"),
    (1024 * 1024, "1m"),
];
const TEST_FILE_SIZE: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    SeqRead,
    SeqWrite,
    RandRead,
    RandWrite,
    RandRW,
}

// impl Mode {
//     pub fn label(&self) -> &'static str {
//         match self {
//             Mode::SeqRead => "顺序读",
//             Mode::SeqWrite => "顺序写",
//             Mode::RandRead => "随机读",
//             Mode::RandWrite => "随机写",
//             Mode::RandRW => "混合读写",
//         }
//     }
// }

// impl Mode {
//     fn label(&self) -> &'static str {
//         match self {
//             Mode::Seq => "顺序读写",
//             Mode::RandRW => "随机读写",
//             Mode::SeqRead => "顺序读",
//             Mode::SeqWrite => "顺序写",
//             Mode::RandRead => "随机读",
//             Mode::RandWrite => "随机写",
//         }
//     }
// }

fn bench_rw(block_size: usize, mode: Mode, concurrency: usize, o_direct: bool, rw_mix: f64) -> (f64, f64, f64, f64) {
    let file_size = TEST_FILE_SIZE;
    let blocks = file_size / block_size;
    let testfile_path = "./testfile_benchio";
    // 先创建大文件，避免多线程竞争
    {
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(testfile_path).expect("无法创建测试文件");
        let buf = vec![0u8; block_size];
        for _ in 0..blocks {
            file.write_all(&buf).unwrap();
        }
        file.sync_all().unwrap();
    }

    let barrier = Arc::new(Barrier::new(concurrency));
    let read_bytes = Arc::new(Mutex::new(0f64));
    let write_bytes = Arc::new(Mutex::new(0f64));
    let read_time = Arc::new(Mutex::new(0f64));
    let write_time = Arc::new(Mutex::new(0f64));

    let mut handles = vec![];
    for tid in 0..concurrency {
        let barrier = barrier.clone();
        let read_bytes = Arc::clone(&read_bytes);
        let write_bytes = Arc::clone(&write_bytes);
        let read_time = Arc::clone(&read_time);
        let write_time = Arc::clone(&write_time);
        let path = testfile_path.to_string();
        let mode = mode;
        let handle = thread::spawn(move || {
            let mut openopt = OpenOptions::new();
            openopt.read(true).write(true);
            if o_direct {
                openopt.custom_flags(libc::O_DIRECT);
            }
            let mut file = openopt.open(&path).unwrap_or_else(|_| {
                // O_DIRECT 失败降级
                let mut fallback = OpenOptions::new();
                fallback.read(true).write(true);
                fallback.open(&path).expect("线程无法打开文件")
            });
            let blocks_per_thread = blocks / concurrency;
            let start_offset = tid * blocks_per_thread * block_size;
            let mut rng = rand::rng();
            let buf = vec![0u8; block_size];
            let mut read_buf = vec![0u8; block_size];
            barrier.wait();

            // Write/Read/混合
            let mut wtime = 0.0;
            let mut rtime = 0.0;
            let mut wbytes = 0.0;
            let mut rbytes = 0.0;
            match mode {
                Mode::SeqWrite => {
                    let start = Instant::now();
                    for i in 0..blocks_per_thread {
                        let offset = (start_offset + i * block_size) as u64;
                        file.seek(SeekFrom::Start(offset)).unwrap();
                        file.write_all(&buf).unwrap();
                    }
                    file.sync_all().unwrap();
                    wtime = start.elapsed().as_secs_f64();
                    wbytes = (blocks_per_thread * block_size) as f64;
                }
                Mode::SeqRead => {
                    let start = Instant::now();
                    for i in 0..blocks_per_thread {
                        let offset = (start_offset + i * block_size) as u64;
                        file.seek(SeekFrom::Start(offset)).unwrap();
                        file.read_exact(&mut read_buf).unwrap();
                    }
                    rtime = start.elapsed().as_secs_f64();
                    rbytes = (blocks_per_thread * block_size) as f64;
                }
                Mode::RandWrite => {
                    let start = Instant::now();
                    for _ in 0..blocks_per_thread {
                        let offset = rng.random_range(0..blocks) as u64 * block_size as u64;
                        file.seek(SeekFrom::Start(offset)).unwrap();
                        file.write_all(&buf).unwrap();
                    }
                    file.sync_all().unwrap();
                    wtime = start.elapsed().as_secs_f64();
                    wbytes = (blocks_per_thread * block_size) as f64;
                }
                Mode::RandRead => {
                    let start = Instant::now();
                    for _ in 0..blocks_per_thread {
                        let offset = rng.random_range(0..blocks) as u64 * block_size as u64;
                        file.seek(SeekFrom::Start(offset)).unwrap();
                        file.read_exact(&mut read_buf).unwrap();
                    }
                    rtime = start.elapsed().as_secs_f64();
                    rbytes = (blocks_per_thread * block_size) as f64;
                }
                Mode::RandRW => {
                    let start = Instant::now();
                    for _ in 0..blocks_per_thread {
                        if rng.random_bool(rw_mix) {
                            let offset = rng.random_range(0..blocks) as u64 * block_size as u64;
                            file.seek(SeekFrom::Start(offset)).unwrap();
                            file.read_exact(&mut read_buf).unwrap();
                            rbytes += block_size as f64;
                        } else {
                            let offset = rng.random_range(0..blocks) as u64 * block_size as u64;
                            file.seek(SeekFrom::Start(offset)).unwrap();
                            file.write_all(&buf).unwrap();
                            wbytes += block_size as f64;
                        }
                    }
                    file.sync_all().unwrap();
                    let elapsed = start.elapsed().as_secs_f64();
                    // 按比例分配时间
                    rtime = elapsed * rw_mix;
                    wtime = elapsed * (1.0 - rw_mix);
                }
            }
            *write_time.lock().unwrap() += wtime;
            *write_bytes.lock().unwrap() += wbytes;
            *read_time.lock().unwrap() += rtime;
            *read_bytes.lock().unwrap() += rbytes;
        });
        handles.push(handle);
    }
    for h in handles {
        h.join().unwrap();
    }

    // 测试结束后删除文件
    let _ = std::fs::remove_file(testfile_path);

    let read_bytes = *read_bytes.lock().unwrap();
    let write_bytes = *write_bytes.lock().unwrap();
    let read_time = *read_time.lock().unwrap() / concurrency as f64;
    let write_time = *write_time.lock().unwrap() / concurrency as f64;
    (read_bytes, read_time, write_bytes, write_time)
}

// 已弃用
// fn format_speed(bytes_per_sec: f64) -> String {
//     if bytes_per_sec > 1024.0 * 1024.0 * 1024.0 {
//         format!("{:.2} GB/s", bytes_per_sec / 1024.0 / 1024.0 / 1024.0)
//     } else if bytes_per_sec > 1024.0 * 1024.0 {
//         format!("{:.2} MB/s", bytes_per_sec / 1024.0 / 1024.0)
//     } else if bytes_per_sec > 1024.0 {
//         format!("{:.2} KB/s", bytes_per_sec / 1024.0)
//     } else {
//         format!("{:.2} B/s", bytes_per_sec)
//     }
// }

// fn format_iops(iops: f64) -> String {
//     if iops > 10_000.0 {
//         format!("{:.1}k", iops / 1000.0)
//     } else {
//         format!("{:.0}", iops)
//     }
// }

pub fn run_io_test() {
    let concurrency = 4;
    let modes = [
        (Mode::SeqRead, "顺序读"),
        (Mode::SeqWrite, "顺序写"),
        (Mode::RandRead, "随机读"),
        (Mode::RandWrite, "随机写"),
        (Mode::RandRW, "混合读写(50%/50%)"),
    ];
    let o_directs = [(false, "缓存IO"), (true, "O_DIRECT(绕过缓存)")];
    for &(o_direct, olabel) in &o_directs {
        println!("\n================ {} ================", olabel);
        for &(mode, mlabel) in &modes {
            println!("\n--- {} ---", mlabel);
            println!("Block Size |   4k (MB/s, IOPS)   |  64k (MB/s, IOPS)   | 512k (MB/s, IOPS) |   1m (MB/s, IOPS)");
            println!("-----------|----------------------|---------------------|-------------------|-------------------");
            print!("Result     |");
            for &(size, _) in SIZES.iter() {
                let (read_bytes, read_time, write_bytes, write_time) = bench_rw(size, mode, concurrency, o_direct, 0.5);
                let mbps = (read_bytes + write_bytes) / (read_time + write_time).max(0.0001) / 1024.0 / 1024.0;
                let iops = (read_bytes + write_bytes) / size as f64 / (read_time + write_time).max(0.0001);
                print!(" {:>7.2} MB/s, {:>7.0} |", mbps, iops);
            }
            println!();
        }
    }
    println!("\n说明：每种模式均为多线程并发，单位已标注，O_DIRECT为物理IO，缓存IO为系统缓存加速。\n");
    println!("按任意键返回...");
    let _ = crossterm::event::read();
}
