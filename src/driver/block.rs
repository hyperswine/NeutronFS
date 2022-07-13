// Block driver backend

use core::sync::atomic::AtomicBool;

use super::{ClusterData, ClusterNumber, PAGE_SIZE};
use alloc::{sync::Arc, vec::Vec};

pub type Block = [u8; 4096];

// -------------
// BLOCK DRIVER
// -------------

// USE A RING BUFFER, basically a pointer to the current head and a max size
// Then "push" or append to the buffer at head - 1/curr_size. If would overflow, just replace the head

#[derive(Debug)]
pub struct ReadQueue<'a> {
    queue: Vec<(ClusterNumber, &'a mut [u8], Arc<AtomicBool>)>,
}

#[derive(Debug)]
pub struct WriteQueue {
    queue: Vec<(ClusterNumber, Block, Arc<AtomicBool>)>,
}

impl WriteQueue {
    pub fn new(queue: Vec<(ClusterNumber, Block, Arc<AtomicBool>)>) -> Self {
        Self { queue }
    }
    pub fn push(&mut self, cluster_number: ClusterNumber, block: Block, arc: Arc<AtomicBool>) {
        self.queue.push((cluster_number, block, arc));
    }
    pub fn pop(&mut self) -> Option<(ClusterNumber, Block, Arc<AtomicBool>)> {
        self.queue.pop()
    }
}

impl<'a> ReadQueue<'a> {
    pub fn new(queue: Vec<(ClusterNumber, &'a mut [u8], Arc<AtomicBool>)>) -> Self {
        Self { queue }
    }
    pub fn push(&mut self, buf: &'a mut [u8], cluster_number: ClusterNumber, arc: Arc<AtomicBool>) {
        self.queue.push((cluster_number, buf, arc));
    }
    pub fn pop(&mut self) -> Option<(ClusterNumber, &'a mut [u8], Arc<AtomicBool>)> {
        self.queue.pop()
    }
}

// *mut is not threadsafe
// &mut is not threadsafe
// must use ARC with mutex

/// Interface for block drivers to implement
pub trait BlockDriver<'a> {
    fn push_read_request(
        &mut self,
        buf: &'a mut [u8],
        cluster_number: u64,
        complete: Arc<AtomicBool>,
    );
    fn push_write_request(&mut self, cluster_number: u64, block: Block, complete: Arc<AtomicBool>);
}

// -------------
// NEUTRON-LIKE
// -------------

pub struct NeutronDriver<'a> {
    read_queue: ReadQueue<'a>,
    write_queue: WriteQueue,
}

impl<'a> BlockDriver<'a> for NeutronDriver<'a> {
    fn push_read_request(
        &mut self,
        buf: &'a mut [u8],
        cluster_number: u64,
        complete: Arc<AtomicBool>,
    ) {
        self.read_queue.push(buf, cluster_number, complete);
    }

    fn push_write_request(&mut self, cluster_number: u64, block: Block, complete: Arc<AtomicBool>) {
        self.write_queue.push(cluster_number, block, complete);
    }
}
