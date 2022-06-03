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
#[derive(Debug)]
pub struct ReadQueue<'a> {
    // want to read these blocks
    queue: Vec<(ClusterNumber, &'a mut [u8])>,
}

#[derive(Debug)]
pub struct WriteQueue {
    // want to read these blocks
    // should just clone() it
    queue: Vec<(ClusterNumber, Block)>,
}

impl WriteQueue {
    pub fn new(queue: Vec<(ClusterNumber, Block)>) -> Self {
        Self { queue }
    }
    pub fn push(&mut self, cluster_number: ClusterNumber, block: Block) {
        self.queue.push((cluster_number, block));
    }
    pub fn pop(&mut self) -> Option<(ClusterNumber, Block)> {
        self.queue.pop()
    }
}

impl<'a> ReadQueue<'a> {
    pub fn new(queue: Vec<(ClusterNumber, &'a mut [u8])>) -> Self {
        Self { queue }
    }

    pub fn push(&mut self, buf: &'a mut [u8], cluster_number: ClusterNumber) {
        self.queue.push((cluster_number, buf));
    }
    pub fn pop(&mut self) -> Option<(ClusterNumber, &'a mut [u8])> {
        self.queue.pop()
    }
}

pub trait BlockDriver<'a> {
    fn push_read_request(&mut self, buf: &'a mut [u8], cluster_number: u64);
    fn push_write_request(&mut self, cluster_number: u64, block: Block);
}

// -------------
// NEUTRON-LIKE
// -------------

// neutronapi like implementation

// would also have the actual ACPI or MMIO fields to read/write from
// with a core::ptr::read/write volatile
pub struct NeutronDriver<'a> {
    read_queue: ReadQueue<'a>,
    write_queue: WriteQueue,
}

impl<'a> BlockDriver<'a> for NeutronDriver<'a> {
    fn push_read_request(&mut self, buf: &'a mut [u8], cluster_number: u64) {
        self.read_queue.push(buf, cluster_number);
    }

    fn push_write_request(&mut self, cluster_number: u64, block: Block) {
        self.write_queue.push(cluster_number, block);
    }
}
