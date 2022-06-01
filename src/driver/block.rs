// Block driver backend
// a block driver should implement a backend, like an SSD or just a regular file on the host
// the block driver makes read/write requests

use super::{ClusterData, ClusterNumber, PAGE_SIZE};
use alloc::vec::Vec;

pub type Block = [u8; 4096];

// -------------
// BLOCK DRIVER
// -------------

// or maybe just have a more generic transaction queue for both reading and writing
// the open file descriptors are handled elsewhere and can be pushed here as an arg

pub struct ReadQueue {
    // want to read these blocks
    queue: Vec<ClusterNumber>,
}

pub struct WriteQueue {
    // want to read these blocks
    queue: Vec<(ClusterNumber, Block)>,
}

impl WriteQueue {
    pub fn new(queue: Vec<(ClusterNumber, Block)>) -> Self {
        Self { queue }
    }
    pub fn push(&mut self, cluster_number: ClusterNumber, block: Block) {
        self.queue.push((cluster_number, block));
    }
}

impl ReadQueue {
    pub fn new(queue: Vec<ClusterNumber>) -> Self {
        Self { queue }
    }
    pub fn push(&mut self, cluster_number: ClusterNumber) {
        self.queue.push(cluster_number);
    }
}

pub trait BlockDriver {
    fn push_read_request(&mut self, cluster_number: u64);
    fn push_write_request(&mut self, cluster_number: u64, block: Block);
}

// -------------
// NEUTRON-LIKE
// -------------

// neutronapi like implementation

// would also have the actual ACPI or MMIO fields to read/write from
// with a core::ptr::read/write volatile
pub struct NeutronDriver {
    read_queue: ReadQueue,
    write_queue: WriteQueue,
}

impl BlockDriver for NeutronDriver {
    fn push_read_request(&mut self, cluster_number: u64) {
        self.read_queue.push(cluster_number);
    }

    fn push_write_request(&mut self, cluster_number: u64, block: Block) {
        self.write_queue.push(cluster_number, block);
    }
}
