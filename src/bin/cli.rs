// Similar to the qfs.elf program, but with NeFS/Neutron specific functionalities
// Callable within the kernel. QFS functionalities are callable within arcboot

use clap::Parser;
use core::task;
use neutron_fs::driver::block::{Block, BlockDriver, ReadQueue, WriteQueue};
use std::cell::RefCell;
use std::io::Write;
use std::ops::DerefMut;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::{fs::File, ptr::null};
use std::{thread, time};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    name: String,
    #[clap(short, long, default_value_t = 1)]
    count: u8,
}

use pasts::{prelude::*, Loop, Task};

/// The nefs.elf program
fn main() {
    // let args = Args::parse();

    // for _ in 0..args.count {
    //     println!("Hello {}!", args.name)
    // }

    // let filepath = args.name.clone();

    // let mut f = File::open(filepath).expect("Couldnt open file");

    // parse the header
}

fn simulate() -> ! {
    // build a sample partition with 100 blocks
    // to use static mut, you need non const
    // prob better to see how Arc works?

    let mut blocks: Vec<Block> = vec![[0 as u8; 4096]; 100];
    let mut read_queue: Mutex<ReadQueue> = Mutex::new(ReadQueue::new(vec![]));
    let mut write_queue: Mutex<WriteQueue> = Mutex::new(WriteQueue::new(vec![]));
    let mut vpartition: VPartition = VPartition::new(100, blocks, read_queue, write_queue);
    let mut partition = Arc::new(Mutex::new(vpartition));

    // partition has 'a lifetime
    // function may have a longer lifetime or something

    println!(
        "created a virtual partition of 100 blocks. Partition = {:?}",
        partition
    );

    // let mut buf = Vec::with_capacity(4096);
    static mut buf: [u8; 4096] = [0 as u8; 4096];
    let cluster_number = 1;

    let t = thread::spawn(move || {
        unsafe {
            loop {
                // try lock
                let mut lock = partition.try_lock();
                if let Ok(ref mut mutex) = lock {
                    let part = mutex.deref_mut();
                    part.handle_requests();
                }
            }
        }
    });
    t.join().expect("The listener thread panicked");

    // make read request
    // ? why doesnt it work? I have it on Arc
    let read_req = thread::spawn(move || unsafe {
        loop {
            // try lock
            let mut lock = partition.try_lock();
            let mut complete: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
            if let Ok(ref mut mutex) = lock {
                let part = mutex.deref_mut();
                part.push_read_request(&mut buf, cluster_number, complete);
            }
        }
    });
    read_req.join().expect("Request made");

    // make a write request

    // make another write request

    // print out results
    // NOTE: could also busy wait for flag = true for a specific request

    // kernel idle loop
    loop {}
}

fn get_help_message() -> String {
    "
    cli
        -r <file> \t[read a file]
        -b        \t[build a struct]
        -o <file> \t[output an in memory struct to a file]
    "
    .to_string()
}

// -------------
// VIRTUAL PARTITION
// -------------

// host file backend
// sample implementation

// a drive has >= 1 partition and is formatted with GPT
// we only care about the partition itself

// In memory view of a partition
// Contains number of blocks and block size
// All blocks are always in order
// Assume you cant resize a partition

// idk about interrupts
// i want async or something
// maybe we impl send/sync

/// You can only read a single block at a time and write single block at a time
/// In and Out requests are queued in its readqueue/writequeue
/// n_blocks, blocks can only be accessed once at a time
/// Once a request is fulfilled completely, the calling thread will be signalled. If a read() req, the buffer should be filled
#[derive(Debug)]
pub struct VPartition {
    n_blocks: u64,
    blocks: Vec<Block>,
    // wrap the read queue in a semaphore since two threads are on it almost at the same time
    // maybe have a handle_read and handle_write
    // it makes sense to busy wait. Or maybe just poll it every 1 second
    // yeah that makes sense. Every X seconds, poll the read and write queue. If there is something, do it. The problem is race conditions then. You need a lock
    read_queue: Mutex<ReadQueue<'static>>,
    write_queue: Mutex<WriteQueue>,
}

impl VPartition {
    pub fn new(
        n_blocks: u64,
        blocks: Vec<Block>,
        read_queue: Mutex<ReadQueue<'static>>,
        write_queue: Mutex<WriteQueue>,
    ) -> Self {
        Self {
            n_blocks,
            blocks,
            read_queue,
            write_queue,
        }
    }

    // dont use async unless impl Future/you want something to return
    // maybe async for the trait
    // ? &'a self or &self ?
    pub fn handle_requests(&mut self) {
        // NOTE: dont have separate reads and writes as that may cause some race conditions between what we need
        // on disk. IDK actually. maybe we want to prioritise a read req. To have the latest data. But its hard to know what the user really wants. Thats why in memory is much better
        // disk should just be read and written in any order prob. Its hard to control since theres so many variables
        // and we dont have our own design of hardware

        let mut file = File::create("output.txt").unwrap();
        file.write(b"This is an output");

        // MAYBE stdout for this thread not the same?
        println!("In handler function!");
        loop {
            println!("In loop!!");

            // check read requests
            let mut lock = self.read_queue.try_lock();
            if let Ok(ref mut mutex) = lock {
                // handle read request by accessing disk
                // NOTE: you would prob need another lock for this if multithreaded drivers/handlers
                // where multiple handler functions can be spawned for each syscall
                // just need to lock blocks
                let read_queue = mutex.deref_mut();
                let block_number = read_queue.pop();
                match block_number {
                    Some(b) => {
                        println!("Read request for block number: {}", b.0);
                        let blck = b.0;
                        let res = self.blocks.get(blck as usize);
                        match res {
                            Some(r) => {
                                b.1.copy_from_slice(r);
                                // you can actually continue from read_queue.pop() again until theres no more read req
                                // but that might starve the write reqs

                                // DEBUG: copy the bytes out to file
                                file.write(b.1);
                            }
                            None => {
                                println!("BLOCK NUMBER INVALID... PANICKING");
                                panic!("Panicked thread");
                            }
                        }
                    }
                    None => println!("No pending reads, checking write"),
                }
            } else {
                println!("read lock is being used, trying write...");
            }
            // check write requests
            if let Ok(ref mut mutex) = self.write_queue.try_lock() {
                // write the block to RAM
                // can just move it because you dont need it anymore
                let write_queue = mutex.deref_mut();
                let block_to_write = write_queue.pop();
                match block_to_write {
                    Some(btw) => {
                        match self.blocks.get_mut(btw.0 as usize) {
                            Some(b) => {
                                // move it instead. I think you can use move from data or std::mem::move
                                b.copy_from_slice(&btw.1);
                            }
                            None => todo!(),
                        }
                    }
                    None => {
                        println!("No pending writes");
                    }
                }
            } else {
                println!("write lock is being used... nothing to do");
            }

            println!("Iteration done. Continuing again in 0.3 sec...");
            thread::sleep(time::Duration::from_millis(30));
        }
    }
}

impl<'a> BlockDriver<'a> for VPartition {
    // calls wake() on handle_requests thread
    fn push_read_request(
        &mut self,
        buf: &'a mut [u8],
        cluster_number: u64,
        complete: Arc<AtomicBool>,
    ) {
        loop {
            let mut lock = self.read_queue.try_lock();
            if let Ok(ref mut mutex) = lock {
                mutex.push(buf, cluster_number, complete);
                break;
            } else {
                println!("lock is being used, trying again in 0.1 sec...");
                thread::sleep(time::Duration::from_millis(10));
            }
        }
        // either sleep here until handler signals this thread
        // or pass a buf

        // self.read_queue.push(cluster_number);
    }

    fn push_write_request(&mut self, cluster_number: u64, block: Block, complete: Arc<AtomicBool>) {
        loop {
            let mut lock = self.write_queue.try_lock();
            if let Ok(ref mut mutex) = lock {
                mutex.push(cluster_number, block, complete);
                break;
            } else {
                println!("lock is being used, trying again in 0.1 sec...");
                thread::sleep(time::Duration::from_millis(10));
            }
        }
    }
}

// -------------
// TESTS
// -------------

#[test]
fn test_stuff() {
    assert_eq!(1, 1);
}