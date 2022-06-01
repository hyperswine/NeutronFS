// Similar to the qfs.elf program, but with NeFS/Neutron specific functionalities
// Callable within the kernel. QFS functionalities are callable within arcboot

use clap::Parser;
use core::task;
use neutron_fs::driver::block::{Block, BlockDriver, ReadQueue, WriteQueue};
use std::{fs::File, ptr::null};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long)]
    name: String,

    /// Number of times to greet
    #[clap(short, long, default_value_t = 1)]
    count: u8,
}

use pasts::{prelude::*, Loop, Task};

/// The nefs.elf program
fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        println!("Hello {}!", args.name)
    }

    let filepath = args.name.clone();

    let mut f = File::open(filepath).expect("Couldnt open file");

    // parse the header
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
pub struct VPartition {
    n_blocks: u64,
    blocks: Vec<Block>,
    read_queue: ReadQueue,
    write_queue: WriteQueue,
}

impl VPartition {
    pub fn new(
        n_blocks: u64,
        blocks: Vec<Block>,
        read_queue: ReadQueue,
        write_queue: WriteQueue,
    ) -> Self {
        Self {
            n_blocks,
            blocks,
            read_queue,
            write_queue,
        }
    }

    // async
    pub async fn handle_requests() {
        // as you get stuff, do it
        // if something in the read or write queue, handle it
        // just sleep for 1 sec then do it
    }
}

impl BlockDriver for VPartition {
    // calls wake() on handle_requests thread
    fn push_read_request(&mut self, cluster_number: u64) {
        self.read_queue.push(cluster_number);
    }

    fn push_write_request(&mut self, cluster_number: u64, block: Block) {
        self.write_queue.push(cluster_number, block);
    }
}
