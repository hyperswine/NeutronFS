use bytes::Bytes;
use core::task;
use neutron_fs::driver::block::{Block, BlockDriver, ReadQueue, WriteQueue};
use std::cell::RefCell;
use std::io::Write;
use std::ops::DerefMut;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::{fs::File, ptr::null};
use std::{thread, time};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};

type Responder<T> = oneshot::Sender<Result<T, &'static str>>;

#[derive(Debug)]
enum DiskRequest {
    Get { key: String },
    Set { key: String, val: Bytes },
}

#[derive(Debug)]
enum DiskReponse {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(64);

    let manager = tokio::spawn(async move {
        // Start receiving messages
        while let Some(cmd) = rx.recv().await {
            match cmd {
                DiskRequest::Get { key } => {
                    client.get(&key).await;
                }
                DiskRequest::Set { key, val } => {
                    client.set(&key, val).await;
                }
            }
        }
    });
}

fn simulate() -> ! {
    let mut blocks: Vec<Block> = vec![[0 as u8; 4096]; 100];
    let mut read_queue: Mutex<ReadQueue> = Mutex::new(ReadQueue::new(vec![]));
    let mut write_queue: Mutex<WriteQueue> = Mutex::new(WriteQueue::new(vec![]));
    let mut vpartition: VPartition = VPartition::new(100, blocks, read_queue, write_queue);
    let mut partition = Arc::new(Mutex::new(vpartition));

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

#[derive(Debug)]
pub struct VPartition {
    n_blocks: u64,
    blocks: Vec<Block>,
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

    pub fn handle_requests(&mut self) {
        
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
