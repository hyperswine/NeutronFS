// Block driver backend
// a block driver should implement a backend, like an SSD or just a regular file on the host
// the block driver makes read/write requests

use super::{ClusterData, ClusterNumber, PAGE_SIZE};
use alloc::vec::Vec;

// -------------
// MEMORY READER
// -------------

// ? I DONT THINK WE NEED THIS

/// A type that impls Read should be a file like type
/// that starts at its starting offset
pub trait Read {
    // e.g. [u8; 4096]. Should be moved across memory or from disk to buffer
    // to dual copy, use .clone()
    type Res;

    // a source S, which is either a trait or something else
    // defaultly used function
    fn read_from_source(&self, offset: u64, n_bytes: u64) -> Self::Res;
}

pub struct MemReader4096;

impl Read for MemReader4096 {
    type Res = [u8; 4096];

    fn read_from_source(&self, offset: u64, n_bytes: u64) -> Self::Res {
        // do a copy() from the source page

        [0 as u8; 4096]
    }
}

// -------------
// BLOCK DRIVER
// -------------

// or maybe just have a more generic transaction queue for both reading and writing
// the open file descriptors are handled elsewhere and can be pushed here as an arg

pub struct TransactionQueue {
    // want to read these blocks
    queue: Vec<ClusterNumber>,
}

impl TransactionQueue {
    pub fn new(queue: Vec<ClusterNumber>) -> Self {
        Self { queue }
    }
    pub fn push(&mut self, cluster_number: ClusterNumber) {
        self.queue.push(cluster_number);
    }
}

pub trait BlockDriver {
    fn push_read_request(&mut self, cluster_number: u64);
    
}

// neutronapi like implementation

pub struct NeutronDriver {
    read_queue: TransactionQueue,
}

impl BlockDriver for NeutronDriver {
    fn push_read_request(&mut self, cluster_number: u64) {
        self.read_queue.push(cluster_number);
    }

    
}

// -------------
// VIRTUAL PARTITION
// -------------

// host file backend
// sample implementation

type Block = [u8; 4096];

// a drive has >= 1 partition and is formatted with GPT
// we only care about the partition itself

// In memory view of a partition
// Contains number of blocks and block size
// All blocks are always in order
// Assume you cant resize a partition

/// You can only read a single block at a time and write single block at a time
/// In and Out requests are queued in its readqueue/writequeue
pub struct Partition {
    n_blocks: u64,
    blocks: Vec<Block>,
    read_queue: TransactionQueue,
    write_queue: TransactionQueue,
}

impl Partition {
    pub fn new(
        n_blocks: u64,
        blocks: Vec<Block>,
        read_queue: TransactionQueue,
        write_queue: TransactionQueue,
    ) -> Self {
        Self {
            n_blocks,
            blocks,
            read_queue,
            write_queue,
        }
    }

    pub fn 
}
